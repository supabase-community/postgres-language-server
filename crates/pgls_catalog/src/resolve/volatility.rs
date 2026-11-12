//! Whether an expression calls a volatile function, like Postgres' [`contain_volatile_functions`]
//! on the expression that parse analysis makes of it.
//!
//! [`contain_volatile_functions`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/optimizer/util/clauses.c#L552

use pgls_query::{NodeEnum, protobuf};
use pgls_query_ext::utils::{QualifiedName, qualified_name};

use super::{expr::infer_expr, resolver::Resolver};
use crate::Behavior;
use crate::lookup::{CatalogView, FunctionInfo, Lookup, OperatorKind};
use crate::typing::{Decision, Type, TypeId, normalize_type_name};

/// How many nodes of an expression [`contains_volatile_functions`] visits. A column default or a
/// `USING` expression has a few dozen; ten thousand only stops generated input, and visiting them
/// takes well under a millisecond.
const VOLATILITY_NODES_MAX: usize = 10_000;

/// Objects with an oid below `FirstNormalObjectId` come with Postgres. None of the built-in
/// operators, casts and type input or output functions is volatile (`pg_operator.oprcode`,
/// `pg_cast.castfunc`, `pg_type.typinput` and `typoutput`, checked on Postgres 15 and 18).
///
/// [`FirstNormalObjectId`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/access/transam.h#L197
const FIRST_NORMAL_OBJECT_ID: i64 = 16384;

/// Whether the raw expression `expr` calls a volatile function: [`contain_volatile_functions`]
/// on what parse analysis makes of it, which [`ATExecAddColumn`] uses to decide whether a new
/// column's default needs a table rewrite.
///
/// The functions, operators and casts of the expression follow [`check_functions_in_node`]: a
/// function call is volatile if every function it may call is, and not volatile if none of
/// them is and the call passes every argument (Postgres passes the defaults of the others, which
/// may be volatile). Built-in operators and casts are never volatile; the volatility of operators
/// and casts of the user isn't in the catalog. Literals cast to a type become constants while
/// parsing. Anything else, like a sublink or a column, is unknown.
///
/// The planner folds constants first, so `coalesce(1, random())` loses its volatile call. The
/// expression is checked as written, so such calls still count as volatile.
///
/// [`contain_volatile_functions`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/optimizer/util/clauses.c#L552
/// [`ATExecAddColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L7546
/// [`check_functions_in_node`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/nodes/nodeFuncs.c#L1906
pub fn contains_volatile_functions(
    expr: &NodeEnum,
    catalog: &dyn CatalogView,
    search_path: &[String],
) -> Decision<bool> {
    let mut resolver = Resolver::new(catalog, search_path, None, None);
    let mut stack = vec![expr];
    let mut visited = 0;
    let mut unknown = false;
    while let Some(node) = stack.pop() {
        visited += 1;
        if visited > VOLATILITY_NODES_MAX {
            tracing::warn!(
                "stopped checking the volatility of an expression after {VOLATILITY_NODES_MAX} nodes"
            );
            return Decision::Unknown;
        }
        match node_volatility(&mut resolver, node) {
            Decision::Known(true) => return Decision::Known(true),
            Decision::Known(false) => {}
            Decision::Unknown => unknown = true,
        }
        push_children(node, &mut stack);
    }
    if unknown {
        Decision::Unknown
    } else {
        Decision::Known(false)
    }
}

/// Whether the node itself, without its children, calls a volatile function.
fn node_volatility(r: &mut Resolver<'_>, node: &NodeEnum) -> Decision<bool> {
    match node {
        NodeEnum::AConst(_)
        | NodeEnum::SqlvalueFunction(_)
        | NodeEnum::BoolExpr(_)
        | NodeEnum::NullTest(_)
        | NodeEnum::BooleanTest(_)
        | NodeEnum::CoalesceExpr(_)
        | NodeEnum::MinMaxExpr(_)
        | NodeEnum::RowExpr(_)
        | NodeEnum::AArrayExpr(_)
        | NodeEnum::CollateClause(_)
        | NodeEnum::AIndirection(_)
        | NodeEnum::AIndices(_)
        | NodeEnum::String(_)
        | NodeEnum::AStar(_)
        | NodeEnum::NamedArgExpr(_)
        | NodeEnum::CaseWhen(_)
        | NodeEnum::List(_) => Decision::Known(false),
        NodeEnum::FuncCall(call) => call_volatility(r.catalog, call, r.search_path),
        NodeEnum::TypeCast(cast) => cast_volatility(r, cast),
        NodeEnum::AExpr(expr) => a_expr_volatility(r.catalog, expr, r.search_path),
        // `CASE x WHEN y` compares with `=` ([`transformCaseExpr`]).
        //
        // [`transformCaseExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1642
        NodeEnum::CaseExpr(case) => match case.arg {
            Some(_) => {
                operator_volatility(r.catalog, None, "=", OperatorKind::Infix, r.search_path)
            }
            None => Decision::Known(false),
        },
        _ => Decision::Unknown,
    }
}

fn push_children<'a>(node: &'a NodeEnum, stack: &mut Vec<&'a NodeEnum>) {
    let mut push = |child: Option<&'a protobuf::Node>| {
        if let Some(child) = child.and_then(|child| child.node.as_ref()) {
            stack.push(child);
        }
    };
    match node {
        NodeEnum::FuncCall(call) => call.args.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::NamedArgExpr(arg) => push(arg.arg.as_deref()),
        NodeEnum::TypeCast(cast) => push(cast.arg.as_deref()),
        NodeEnum::AExpr(expr) => {
            push(expr.lexpr.as_deref());
            push(expr.rexpr.as_deref());
        }
        NodeEnum::BoolExpr(expr) => expr.args.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::NullTest(test) => push(test.arg.as_deref()),
        NodeEnum::BooleanTest(test) => push(test.arg.as_deref()),
        NodeEnum::CoalesceExpr(expr) => expr.args.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::MinMaxExpr(expr) => expr.args.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::RowExpr(expr) => expr.args.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::AArrayExpr(expr) => expr.elements.iter().for_each(|arg| push(Some(arg))),
        NodeEnum::CollateClause(clause) => push(clause.arg.as_deref()),
        NodeEnum::CaseExpr(case) => {
            push(case.arg.as_deref());
            case.args.iter().for_each(|arg| push(Some(arg)));
            push(case.defresult.as_deref());
        }
        NodeEnum::CaseWhen(when) => {
            push(when.expr.as_deref());
            push(when.result.as_deref());
        }
        NodeEnum::AIndirection(indirection) => {
            push(indirection.arg.as_deref());
            indirection
                .indirection
                .iter()
                .for_each(|item| push(Some(item)));
        }
        NodeEnum::AIndices(indices) => {
            push(indices.lidx.as_deref());
            push(indices.uidx.as_deref());
        }
        NodeEnum::List(list) => list.items.iter().for_each(|item| push(Some(item))),
        _ => {}
    }
}

/// Whether a function call is volatile: `Known(true)` if every function it may call is
/// volatile, `Known(false)` if none is and the call passes all of their arguments.
///
/// The candidates are the visible functions with the name that accept the number of arguments.
/// Without arguments, Postgres calls the function of the first schema of the search path that
/// has one, which hides those of later schemas ([`FuncnameGetCandidates`]).
///
/// [`FuncnameGetCandidates`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1192
fn call_volatility(
    catalog: &dyn CatalogView,
    call: &protobuf::FuncCall,
    search_path: &[String],
) -> Decision<bool> {
    // Aggregates and window functions are not allowed in a default, and calls with `VARIADIC`
    // aren't modelled.
    if call.agg_star
        || call.agg_distinct
        || call.agg_within_group
        || call.func_variadic
        || call.agg_filter.is_some()
        || call.over.is_some()
        || !call.agg_order.is_empty()
    {
        return Decision::Unknown;
    }
    let Some(QualifiedName { schema, name, .. }) = qualified_name(&call.funcname) else {
        return Decision::Unknown;
    };
    let Lookup::Found(functions) = catalog.functions(schema, name, search_path) else {
        return Decision::Unknown;
    };
    let args = call.args.len();
    let mut candidates: Vec<&FunctionInfo> = functions
        .iter()
        .filter(|function| {
            function.min_args <= args && function.max_args.is_none_or(|max| args <= max)
        })
        .collect();
    if args == 0
        && let Some(first) = candidates.first().map(|function| function.schema.clone())
    {
        candidates.retain(|function| function.schema == first);
    }
    if candidates.is_empty() {
        return Decision::Unknown;
    }
    if candidates
        .iter()
        .all(|function| function.volatility == Some(Behavior::Volatile))
    {
        return Decision::Known(true);
    }
    let passes_all_arguments = |function: &FunctionInfo| match function.max_args {
        Some(max) => args == max,
        None => function
            .signature
            .as_ref()
            .is_some_and(|signature| signature.input_defaults == 0),
    };
    if candidates.iter().all(|function| {
        function
            .volatility
            .is_some_and(|volatility| volatility != Behavior::Volatile)
            && passes_all_arguments(function)
    }) {
        Decision::Known(false)
    } else {
        Decision::Unknown
    }
}

/// Whether a cast calls a volatile function. Postgres converts a string literal or `NULL` while
/// parsing, with the input function of the type ([`transformTypeCast`]), and the cast of an
/// empty `ARRAY[]` is its type. Other casts call a function of the cast, or the output and input
/// functions of the types, which are never volatile for built-in types.
///
/// [`transformTypeCast`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2714
fn cast_volatility(r: &mut Resolver<'_>, cast: &protobuf::TypeCast) -> Decision<bool> {
    let Some(arg) = cast.arg.as_deref().and_then(|arg| arg.node.as_ref()) else {
        return Decision::Unknown;
    };
    match arg {
        NodeEnum::AConst(constant)
            if constant.isnull || matches!(constant.val, Some(protobuf::a_const::Val::Sval(_))) =>
        {
            return Decision::Known(false);
        }
        NodeEnum::AArrayExpr(array) if array.elements.is_empty() => {
            return Decision::Known(false);
        }
        _ => {}
    }
    let Some(type_name) = cast.type_name.as_ref() else {
        return Decision::Unknown;
    };
    let (Some(target), _) = normalize_type_name(r.catalog, type_name, r.search_path) else {
        return Decision::Unknown;
    };
    let source = match arg {
        // The source of a cast of a cast is the type of the inner cast.
        NodeEnum::TypeCast(inner) => inner
            .type_name
            .as_ref()
            .and_then(|type_name| normalize_type_name(r.catalog, type_name, r.search_path).0)
            .map(Type::Named),
        arg => infer_expr(r, arg),
    };
    match source {
        Some(Type::Named(source)) if source == target => Decision::Known(false),
        Some(Type::Named(source)) if is_builtin_type(&source) && is_builtin_type(&target) => {
            Decision::Known(false)
        }
        // Expressions of unknown type other than literals become `text`.
        Some(Type::UnknownLiteral) if is_builtin_type(&target) => Decision::Known(false),
        _ => Decision::Unknown,
    }
}

fn is_builtin_type(id: &TypeId) -> bool {
    matches!(id, TypeId::Snapshot(oid) if (1..FIRST_NORMAL_OBJECT_ID).contains(oid))
}

/// Whether the operators of an `A_Expr` are volatile. `BETWEEN` compares with `>=` and `<=`, and
/// `NOT BETWEEN` with `<` and `>` ([`transformAExprBetween`]); the others name their operator,
/// like `=` for `IN`, `IS DISTINCT FROM` and `NULLIF`.
///
/// [`transformAExprBetween`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1294
fn a_expr_volatility(
    catalog: &dyn CatalogView,
    expr: &protobuf::AExpr,
    search_path: &[String],
) -> Decision<bool> {
    use protobuf::AExprKind as Kind;

    let kind = if expr.lexpr.is_some() {
        OperatorKind::Infix
    } else {
        OperatorKind::Prefix
    };
    let both =
        |left: &str, right: &str| match operator_volatility(catalog, None, left, kind, search_path)
        {
            Decision::Known(false) => operator_volatility(catalog, None, right, kind, search_path),
            other => other,
        };
    match Kind::try_from(expr.kind) {
        Ok(Kind::AexprBetween | Kind::AexprBetweenSym) => both(">=", "<="),
        Ok(Kind::AexprNotBetween | Kind::AexprNotBetweenSym) => both("<", ">"),
        Ok(_) => match qualified_name(&expr.name) {
            Some(QualifiedName { schema, name, .. }) => {
                operator_volatility(catalog, schema, name, kind, search_path)
            }
            None => Decision::Unknown,
        },
        Err(_) => Decision::Unknown,
    }
}

/// An operator is not volatile if every operator it may resolve to is built in.
fn operator_volatility(
    catalog: &dyn CatalogView,
    schema: Option<&str>,
    name: &str,
    kind: OperatorKind,
    search_path: &[String],
) -> Decision<bool> {
    let candidates = catalog.operator_candidates(schema, name, kind, search_path);
    if candidates.complete
        && !candidates.items.is_empty()
        && candidates
            .items
            .iter()
            .all(|operator| (1..FIRST_NORMAL_OBJECT_ID).contains(&operator.oid))
    {
        Decision::Known(false)
    } else {
        Decision::Unknown
    }
}

#[cfg(test)]
mod tests {
    use pgls_query::protobuf::{AConst, BoolExpr, BoolExprType, Node, a_const};

    use super::*;
    use crate::Catalog;

    fn volatility(sql: &str) -> Decision<bool> {
        let NodeEnum::SelectStmt(select) = pgls_query::parse(&format!("select {sql}"))
            .unwrap()
            .into_root()
            .unwrap()
        else {
            panic!("not a select");
        };
        let Some(NodeEnum::ResTarget(target)) = select.target_list[0].node.as_ref() else {
            panic!("no target");
        };
        let expr = target.val.as_ref().unwrap().node.as_ref().unwrap();
        contains_volatile_functions(expr, &Catalog::new(None), &[])
    }

    #[test]
    fn literals_without_a_catalog() {
        assert_eq!(volatility("1"), Decision::Known(false));
        assert_eq!(volatility("'a'::my_type"), Decision::Known(false));
        assert_eq!(volatility("NULL::my_type"), Decision::Known(false));
        assert_eq!(
            volatility("coalesce(NULL, 'a'::my_type)"),
            Decision::Known(false)
        );
        assert_eq!(volatility("1::my_type"), Decision::Unknown);
        assert_eq!(volatility("now()"), Decision::Unknown);
        assert_eq!(volatility("1 + 1"), Decision::Unknown);
        assert_eq!(volatility("(select 1)"), Decision::Unknown);
    }

    #[test]
    fn stops_after_the_node_limit() {
        let constant = Node {
            node: Some(NodeEnum::AConst(AConst {
                isnull: false,
                location: 0,
                val: Some(a_const::Val::Boolval(protobuf::Boolean { boolval: true })),
            })),
        };
        let and = |args: usize| {
            NodeEnum::BoolExpr(Box::new(BoolExpr {
                boolop: BoolExprType::AndExpr.into(),
                args: vec![constant.clone(); args],
                ..Default::default()
            }))
        };
        let catalog = Catalog::new(None);
        assert_eq!(
            contains_volatile_functions(&and(VOLATILITY_NODES_MAX - 1), &catalog, &[]),
            Decision::Known(false)
        );
        assert_eq!(
            contains_volatile_functions(&and(VOLATILITY_NODES_MAX), &catalog, &[]),
            Decision::Unknown
        );
    }
}
