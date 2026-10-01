//! Conservative expression type inference used while resolving query output columns.
use super::{
    resolver::Resolver,
    scope::{find_column_type, find_item},
};
use crate::{
    lookup::Lookup,
    typing::{
        CallArg, CoercionContext, Decision, OperatorKind, Selection, Type, can_coerce,
        normalize_type_name, select_common_type, select_function, select_operator,
    },
};
use pgls_query::{NodeEnum, protobuf};

pub(super) fn infer_expr(r: &mut Resolver<'_>, expr: &NodeEnum) -> Option<Type> {
    let named = |r: &Resolver<'_>, name: &str| -> Option<Type> {
        match r.catalog.type_(Some("pg_catalog"), name, r.search_path) {
            Lookup::Found(info) => info.id.map(Type::Named),
            _ => None,
        }
    };
    match expr {
        NodeEnum::ColumnRef(n) => {
            let names = string_values(&n.fields)?;
            match names.as_slice() {
                // A column takes precedence over a whole-row reference to a table.
                [column] => find_column_type(&r.levels, column)
                    .or_else(|| {
                        find_item(&r.levels, column)
                            .and_then(|item| item.typed_columns.clone().map(Type::Record))
                    })
                    .or_else(|| {
                        let param = r.function_param(column)?;
                        function_param_type(r, param)
                    }),
                [table, column] => {
                    find_item(&r.levels, table).and_then(|item| item.type_of(column).flatten())
                }
                _ => None,
            }
        }
        NodeEnum::AConst(n) => {
            use protobuf::a_const::Val;
            if n.isnull {
                return Some(Type::UnknownLiteral);
            }
            match n.val.as_ref()? {
                Val::Ival(_) => named(r, "int4"),
                Val::Fval(f) => {
                    let text = &f.fval;
                    if text.contains('.') || text.contains('e') || text.contains('E') {
                        named(r, "numeric")
                    } else if text.parse::<i64>().is_ok() {
                        named(r, "int8")
                    } else {
                        named(r, "numeric")
                    }
                }
                Val::Boolval(_) => named(r, "bool"),
                Val::Sval(_) => Some(Type::UnknownLiteral),
                Val::Bsval(_) => named(r, "bit"),
            }
        }
        NodeEnum::ParamRef(n) => {
            let param = r
                .function?
                .params
                .get(usize::try_from(n.number).ok()?.checked_sub(1)?)?;
            function_param_type(r, param)
        }
        NodeEnum::TypeCast(n) => {
            let from = n
                .arg
                .as_deref()?
                .node
                .as_ref()
                .and_then(|x| infer_expr(r, x));
            let (id, _) = normalize_type_name(r.catalog, n.type_name.as_ref()?, r.search_path);
            if let (Some(from), Some(to)) = (from, id.as_ref()) {
                if !matches!(from, Type::UnknownLiteral)
                    && matches!(
                        can_coerce(r.catalog, &from, to, CoercionContext::Explicit),
                        Decision::Known(false)
                    )
                {
                    r.report(
                        crate::resolve::FindingKind::InvalidCast {
                            from,
                            to: Type::Named(to.clone()),
                        },
                        n.location,
                    );
                    return None;
                }
            }
            id.map(Type::Named)
        }
        NodeEnum::BoolExpr(_)
        | NodeEnum::NullTest(_)
        | NodeEnum::BooleanTest(_)
        | NodeEnum::DistinctExpr(_)
        | NodeEnum::ScalarArrayOpExpr(_)
        | NodeEnum::RowCompareExpr(_) => named(r, "bool"),
        NodeEnum::NullIfExpr(n) => {
            let left = n
                .args
                .first()?
                .node
                .as_ref()
                .and_then(|x| infer_expr(r, x))?;
            let right = n
                .args
                .get(1)?
                .node
                .as_ref()
                .and_then(|x| infer_expr(r, x))?;
            let _ = select_operator(
                r.catalog,
                None,
                "=",
                OperatorKind::Infix,
                Some(&left),
                &right,
                r.search_path,
            );
            common_types(r, &[left, right])
        }
        NodeEnum::SubLink(n) => match protobuf::SubLinkType::try_from(n.sub_link_type).ok()? {
            protobuf::SubLinkType::ExistsSublink => named(r, "bool"),
            protobuf::SubLinkType::ExprSublink => {
                let query = n.subselect.as_deref()?;
                let columns = super::query_output_types(query, r.catalog, r.search_path)?;
                (columns.len() == 1)
                    .then(|| columns[0].ty.clone())
                    .flatten()
            }
            protobuf::SubLinkType::AnySublink | protobuf::SubLinkType::AllSublink => {
                named(r, "bool")
            }
            _ => None,
        },
        NodeEnum::RowExpr(n) => Some(Type::Record(
            n.args
                .iter()
                .enumerate()
                .map(|(i, arg)| crate::typing::TypedColumn {
                    name: n
                        .colnames
                        .get(i)
                        .and_then(string_value)
                        .unwrap_or("")
                        .to_owned(),
                    ty: arg.node.as_ref().and_then(|x| infer_expr(r, x)),
                })
                .collect(),
        )),
        NodeEnum::SqlvalueFunction(n) => {
            use protobuf::SqlValueFunctionOp as Op;
            let name = match Op::try_from(n.op).ok()? {
                Op::SvfopCurrentDate => "date",
                Op::SvfopCurrentTime | Op::SvfopCurrentTimeN => "timetz",
                Op::SvfopLocaltime | Op::SvfopLocaltimeN => "time",
                Op::SvfopCurrentTimestamp | Op::SvfopCurrentTimestampN => "timestamptz",
                Op::SvfopLocaltimestamp | Op::SvfopLocaltimestampN => "timestamp",
                Op::SvfopCurrentRole
                | Op::SvfopCurrentUser
                | Op::SvfopUser
                | Op::SvfopSessionUser
                | Op::SvfopCurrentCatalog
                | Op::SvfopCurrentSchema => "name",
                _ => return None,
            };
            named(r, name)
        }
        NodeEnum::CollateClause(n) => n
            .arg
            .as_deref()
            .and_then(|x| x.node.as_ref())
            .and_then(|x| infer_expr(r, x)),
        NodeEnum::AIndirection(n) => {
            let mut ty = n
                .arg
                .as_deref()
                .and_then(|x| x.node.as_ref())
                .and_then(|x| infer_expr(r, x))?;
            let mut items = n.indirection.iter().peekable();
            while let Some(item) = items.next() {
                ty = match item.node.as_ref()? {
                    // Consecutive subscripts index one array: `a[1][2]` is an element of a
                    // two-dimensional array, and any slice makes the result an array.
                    NodeEnum::AIndices(indices) => {
                        let mut slice = indices.is_slice;
                        while let Some(NodeEnum::AIndices(next)) =
                            items.peek().and_then(|item| item.node.as_ref())
                        {
                            slice |= next.is_slice;
                            items.next();
                        }
                        let Type::Named(id) = &ty else { return None };
                        let Decision::Known(array) = crate::typing::base_type(r.catalog, id) else {
                            return None;
                        };
                        let Lookup::Found(info) = r.catalog.type_by_id(&array) else {
                            return None;
                        };
                        // Only true arrays: types like jsonb have their own subscripting.
                        let element = info.element?;
                        Type::Named(if slice { array } else { element })
                    }
                    NodeEnum::String(field) => field_type(r, &ty, &field.sval)?,
                    _ => return None,
                };
            }
            Some(ty)
        }
        NodeEnum::AExpr(n) => {
            use protobuf::AExprKind as Kind;
            match Kind::try_from(n.kind).ok()? {
                Kind::AexprIn
                | Kind::AexprOpAny
                | Kind::AexprOpAll
                | Kind::AexprDistinct
                | Kind::AexprNotDistinct
                | Kind::AexprLike
                | Kind::AexprIlike
                | Kind::AexprSimilar
                | Kind::AexprBetween
                | Kind::AexprNotBetween
                | Kind::AexprBetweenSym
                | Kind::AexprNotBetweenSym => return named(r, "bool"),
                Kind::AexprNullif => {
                    let left = n
                        .lexpr
                        .as_deref()?
                        .node
                        .as_ref()
                        .and_then(|x| infer_expr(r, x))?;
                    let right = n
                        .rexpr
                        .as_deref()?
                        .node
                        .as_ref()
                        .and_then(|x| infer_expr(r, x))?;
                    return common_types(r, &[left, right]);
                }
                _ => {}
            }
            let names = string_values(&n.name)?;
            let (schema, name) = match names.as_slice() {
                [name] => (None, name.as_str()),
                [schema, name] => (Some(schema.as_str()), name.as_str()),
                _ => return None,
            };
            let right = infer_expr(r, n.rexpr.as_deref()?.node.as_ref()?)?;
            let left = n
                .lexpr
                .as_deref()
                .and_then(|x| x.node.as_ref())
                .and_then(|x| infer_expr(r, x));
            let kind = if left.is_some() {
                OperatorKind::Infix
            } else {
                OperatorKind::Prefix
            };
            let selection = select_operator(
                r.catalog,
                schema,
                name,
                kind,
                left.as_ref(),
                &right,
                r.search_path,
            );
            match selection {
                Selection::Match(o) => o.result,
                Selection::NoMatch | Selection::Ambiguous => {
                    let failure = if matches!(selection, Selection::NoMatch) {
                        crate::resolve::MatchFailure::NoMatch
                    } else {
                        crate::resolve::MatchFailure::Ambiguous
                    };
                    r.report(
                        crate::resolve::FindingKind::OperatorMismatch {
                            operator: name.to_owned(),
                            left,
                            right,
                            failure,
                        },
                        n.location,
                    );
                    None
                }
                _ => None,
            }
        }
        NodeEnum::FuncCall(n) => {
            let names = string_values(&n.funcname)?;
            let (schema, name) = match names.as_slice() {
                [name] => (None, name.as_str()),
                [schema, name] => (Some(schema.as_str()), name.as_str()),
                _ => return None,
            };
            if n.agg_star {
                return named(r, "int8");
            }
            // `VARIADIC` passes the variadic array itself, and ordered-set aggregates take
            // more arguments than the call lists. Neither is modelled.
            if n.func_variadic || n.agg_within_group {
                return None;
            }
            let args = n
                .args
                .iter()
                .filter_map(|arg| arg.node.as_ref())
                .map(|arg| match arg {
                    NodeEnum::NamedArgExpr(named) => CallArg {
                        ty: named
                            .arg
                            .as_deref()
                            .and_then(|x| x.node.as_ref())
                            .and_then(|x| infer_expr(r, x)),
                        name: Some(named.name.clone()),
                    },
                    arg => CallArg {
                        ty: infer_expr(r, arg),
                        name: None,
                    },
                })
                .collect::<Vec<_>>();
            let selection = select_function(r.catalog, schema, name, &args, r.search_path);
            let failure = match selection {
                Selection::Match(f) => return f.result,
                Selection::NoMatch => crate::resolve::MatchFailure::NoMatch,
                Selection::Ambiguous => crate::resolve::MatchFailure::Ambiguous,
                Selection::Unknown => return None,
            };
            r.report(
                crate::resolve::FindingKind::FunctionArgumentMismatch {
                    schema: schema.map(str::to_owned),
                    name: name.to_owned(),
                    args: args.into_iter().filter_map(|a| a.ty).collect(),
                    failure,
                },
                n.location,
            );
            None
        }
        NodeEnum::CoalesceExpr(n) => common(r, &n.args),
        NodeEnum::MinMaxExpr(n) => common(r, &n.args),
        NodeEnum::CaseExpr(n) => {
            let mut results = Vec::new();
            for when in &n.args {
                let Some(NodeEnum::CaseWhen(when)) = when.node.as_ref() else {
                    return None;
                };
                results.push(
                    when.result
                        .as_deref()
                        .and_then(|x| x.node.as_ref())
                        .and_then(|x| infer_expr(r, x)),
                );
            }
            // Without ELSE, the result is NULL otherwise.
            results.push(match n.defresult.as_deref().and_then(|x| x.node.as_ref()) {
                Some(default) => infer_expr(r, default),
                None => Some(Type::UnknownLiteral),
            });
            common_types(r, &results.into_iter().collect::<Option<Vec<_>>>()?)
        }
        NodeEnum::AArrayExpr(n) => {
            let ty = common(r, &n.elements)?;
            let Type::Named(id) = ty else { return None };
            match r.catalog.type_by_id(&id) {
                Lookup::Found(info) => info.array.map(Type::Named),
                _ => None,
            }
        }
        _ => None,
    }
}
/// The type of a field of a row.
fn field_type(r: &Resolver<'_>, row: &Type, field: &str) -> Option<Type> {
    let columns = match row {
        Type::Record(columns) => columns.clone(),
        Type::Named(id) => {
            let Lookup::Found(info) = r.catalog.type_by_id(id) else {
                return None;
            };
            info.attributes?
                .into_iter()
                .map(|attribute| crate::typing::TypedColumn {
                    name: attribute.name,
                    ty: attribute.ty,
                })
                .collect()
        }
        Type::UnknownLiteral => return None,
    };
    columns.into_iter().find(|column| column.name == field)?.ty
}

fn function_param_type(r: &Resolver<'_>, param: &crate::resolve::FunctionParam) -> Option<Type> {
    let Lookup::Found(mut info) = r.catalog.type_(
        param.type_schema.as_deref(),
        &param.type_name,
        r.search_path,
    ) else {
        return None;
    };
    if param.is_array {
        let element = info.id.as_ref()?;
        let Lookup::Found(array) = r.catalog.type_by_id(element) else {
            return None;
        };
        return array.array.map(Type::Named);
    }
    info.id.take().map(Type::Named)
}

fn string_value(node: &pgls_query::Node) -> Option<&str> {
    match node.node.as_ref()? {
        NodeEnum::String(s) => Some(&s.sval),
        _ => None,
    }
}
fn string_values(nodes: &[pgls_query::Node]) -> Option<Vec<String>> {
    nodes
        .iter()
        .map(|n| string_value(n).map(str::to_owned))
        .collect()
}
/// The common type of the arguments, if all their types are known.
fn common(r: &mut Resolver<'_>, args: &[pgls_query::Node]) -> Option<Type> {
    let types = args
        .iter()
        .map(|arg| arg.node.as_ref().and_then(|x| infer_expr(r, x)))
        .collect::<Vec<_>>();
    common_types(r, &types.into_iter().collect::<Option<Vec<_>>>()?)
}
pub(super) fn check_assignment(
    r: &mut Resolver<'_>,
    column: &str,
    column_type: Type,
    expr_type: Option<Type>,
    location: i32,
) {
    let Some(expr_type) = expr_type else { return };
    if matches!(expr_type, Type::UnknownLiteral) {
        return;
    }
    let Type::Named(target) = &column_type else {
        return;
    };
    if matches!(
        can_coerce(r.catalog, &expr_type, target, CoercionContext::Assignment),
        Decision::Known(false)
    ) {
        r.report(
            crate::resolve::FindingKind::AssignmentMismatch {
                column: column.to_owned(),
                column_type,
                expr_type,
            },
            location,
        );
    }
}

fn common_types(r: &Resolver<'_>, ts: &[Type]) -> Option<Type> {
    match select_common_type(r.catalog, ts) {
        Selection::Match(t) => Some(t),
        _ => None,
    }
}
