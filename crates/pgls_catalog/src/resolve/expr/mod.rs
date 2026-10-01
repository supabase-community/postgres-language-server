//! Conservative expression type inference used while resolving query output columns.
use super::{
    resolver::Resolver,
    scope::{find_column_type, find_item},
};
use crate::{
    lookup::Lookup,
    typing::{
        CallArg, OperatorKind, Selection, Type, normalize_type_name, select_common_type,
        select_function, select_operator,
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
                [table] if find_item(&r.levels, table).is_some() => find_item(&r.levels, table)
                    .and_then(|item| item.typed_columns.clone().map(Type::Record)),
                [column] => find_column_type(&r.levels, column).or_else(|| {
                    let param = r.function_param(column)?;
                    match r.catalog.type_(
                        param.type_schema.as_deref(),
                        &param.type_name,
                        r.search_path,
                    ) {
                        Lookup::Found(info) => info.id.map(Type::Named),
                        _ => None,
                    }
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
            match r.catalog.type_(
                param.type_schema.as_deref(),
                &param.type_name,
                r.search_path,
            ) {
                Lookup::Found(info) => info.id.map(Type::Named),
                _ => None,
            }
        }
        NodeEnum::TypeCast(n) => {
            let (id, _) = normalize_type_name(r.catalog, n.type_name.as_ref()?, r.search_path);
            id.map(Type::Named)
        }
        NodeEnum::BoolExpr(_) | NodeEnum::NullTest(_) | NodeEnum::BooleanTest(_) => {
            named(r, "bool")
        }
        NodeEnum::AExpr(n) => {
            let name = n.name.iter().find_map(|n| match n.node.as_ref()? {
                NodeEnum::String(s) => Some(s.sval.as_str()),
                _ => None,
            })?;
            let right = infer_expr(r, n.rexpr.as_deref()?.node.as_ref()?)?;
            let left = n
                .lexpr
                .as_deref()
                .and_then(|x| x.node.as_ref())
                .and_then(|x| infer_expr(r, x));
            match select_operator(
                r.catalog,
                None,
                name,
                if left.is_some() {
                    OperatorKind::Infix
                } else {
                    OperatorKind::Prefix
                },
                left.as_ref(),
                &right,
                r.search_path,
            ) {
                Selection::Match(o) => o.result,
                _ => None,
            }
        }
        NodeEnum::FuncCall(n) => {
            let name = n.funcname.last().and_then(|x| string_value(x))?;
            let args = n
                .args
                .iter()
                .map(|a| CallArg {
                    ty: a.node.as_ref().and_then(|x| infer_expr(r, x)),
                    name: None,
                })
                .collect::<Vec<_>>();
            match select_function(r.catalog, None, name, &args, r.search_path) {
                Selection::Match(f) => f.result,
                _ => None,
            }
        }
        NodeEnum::CoalesceExpr(n) => common(r, &n.args),
        NodeEnum::MinMaxExpr(n) => common(r, &n.args),
        NodeEnum::CaseExpr(n) => {
            let mut args = n
                .args
                .iter()
                .filter_map(|a| a.node.as_ref())
                .filter_map(|a| match a {
                    NodeEnum::CaseWhen(w) => w
                        .result
                        .as_deref()?
                        .node
                        .as_ref()
                        .and_then(|x| infer_expr(r, x)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if let Some(v) = n
                .defresult
                .as_deref()
                .and_then(|x| x.node.as_ref())
                .and_then(|x| infer_expr(r, x))
            {
                args.push(v)
            };
            common_types(r, &args)
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
fn common(r: &mut Resolver<'_>, args: &[pgls_query::Node]) -> Option<Type> {
    let ts = args
        .iter()
        .filter_map(|a| a.node.as_ref())
        .filter_map(|x| infer_expr(r, x))
        .collect::<Vec<_>>();
    common_types(r, &ts)
}
fn common_types(r: &Resolver<'_>, ts: &[Type]) -> Option<Type> {
    match select_common_type(r.catalog, ts) {
        Selection::Match(t) => Some(t),
        _ => None,
    }
}
