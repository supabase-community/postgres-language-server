//! Conservative expression type inference used while resolving query output columns. Each
//! module follows the `transform*` function of [`transformExprRecurse`] for its node.
//!
//! [`transformExprRecurse`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L137
mod a_array_expr;
mod a_const;
mod a_expr;
mod a_indirection;
mod assignment;
mod bool_expr;
mod boolean_test;
mod case_expr;
mod coalesce_expr;
mod collate_clause;
mod column_ref;
mod func_call;
mod min_max_expr;
mod null_test;
mod param_ref;
mod row_expr;
mod sql_value_function;
mod sub_link;
mod type_cast;
use super::{
    resolver::Resolver,
    scope::{find_column_type, find_item},
};
use crate::{
    OperatorKind,
    lookup::Lookup,
    typing::{
        CallArg, CoercionContext, Decision, Selection, Type, TypeId, can_coerce,
        normalize_type_name, select_common_type, select_function, select_operator,
    },
};
pub(super) use assignment::check_assignment;
pub(super) use func_call::{call_args, resolve_procedure_call};
use pgls_query::{NodeEnum, protobuf};

fn named(r: &Resolver<'_>, name: &str) -> Option<Type> {
    match r.catalog.type_(Some("pg_catalog"), name, r.search_path) {
        Lookup::Found(info) => info.id.map(Type::Named),
        _ => None,
    }
}

pub(super) fn infer_expr(r: &mut Resolver<'_>, expr: &NodeEnum) -> Option<Type> {
    match expr {
        NodeEnum::ColumnRef(n) => column_ref::infer_column_ref(r, n),
        NodeEnum::AConst(n) => a_const::infer_a_const(r, n),
        NodeEnum::ParamRef(n) => param_ref::infer_param_ref(r, n),
        NodeEnum::TypeCast(n) => type_cast::infer_type_cast(r, n),
        NodeEnum::BoolExpr(n) => bool_expr::infer_bool_expr(r, n),
        NodeEnum::NullTest(n) => null_test::infer_null_test(r, n),
        NodeEnum::BooleanTest(n) => boolean_test::infer_boolean_test(r, n),
        NodeEnum::SubLink(n) => sub_link::infer_sub_link(r, n),
        NodeEnum::RowExpr(n) => row_expr::infer_row_expr(r, n),
        NodeEnum::SqlvalueFunction(n) => sql_value_function::infer_sql_value_function(r, n),
        NodeEnum::CollateClause(n) => collate_clause::infer_collate_clause(r, n),
        NodeEnum::AIndirection(n) => a_indirection::infer_a_indirection(r, n),
        NodeEnum::AExpr(n) => a_expr::infer_a_expr(r, n),
        NodeEnum::FuncCall(n) => func_call::infer_func_call(r, n),
        NodeEnum::CoalesceExpr(n) => coalesce_expr::infer_coalesce_expr(r, n),
        NodeEnum::CaseExpr(n) => case_expr::infer_case_expr(r, n),
        NodeEnum::AArrayExpr(n) => a_array_expr::infer_a_array_expr(r, n),
        NodeEnum::MinMaxExpr(n) => min_max_expr::infer_min_max_expr(r, n),
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

/// The type of a parameter in the body of a SQL function. Postgres parses the body when the
/// function is called, with polymorphic parameters (`anyelement`, `anyarray`, ...) resolved to
/// the types of the call's arguments ([`prepare_sql_fn_parse_info`]), so their type in the body is
/// unknown. Other pseudo-types are not allowed for parameters of SQL functions
/// ([`fmgr_sql_validator`]).
///
/// [`prepare_sql_fn_parse_info`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/executor/functions.c#L286
/// [`fmgr_sql_validator`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/pg_proc.c#L873
fn function_param_type(r: &Resolver<'_>, param: &crate::resolve::FunctionParam) -> Option<Type> {
    let Lookup::Found(mut info) = r.catalog.type_(
        param.type_schema.as_deref(),
        &param.type_name,
        r.search_path,
    ) else {
        return None;
    };
    if info.kind == Some(crate::TypeKind::Pseudo) {
        return None;
    }
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
/// The common type of the arguments, if all their types are known ([`select_common_type`]).
///
/// [`select_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1345
fn common(r: &mut Resolver<'_>, args: &[pgls_query::Node]) -> Option<Type> {
    let types = args
        .iter()
        .map(|arg| arg.node.as_ref().and_then(|x| infer_expr(r, x)))
        .collect::<Vec<_>>();
    common_types(r, &types.into_iter().collect::<Option<Vec<_>>>()?)
}
fn common_types(r: &Resolver<'_>, ts: &[Type]) -> Option<Type> {
    match select_common_type(r.catalog, ts) {
        Selection::Match(t) => Some(t),
        _ => None,
    }
}

/// Checks the `=` that `IS DISTINCT FROM`, `NULLIF` and simple `CASE` compare with, like
/// [`make_op`] (through [`oper`]) does. Reports only proven selection failures. Returns the
/// selected operator.
///
/// [`make_op`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L660
/// [`oper`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L370
fn check_operator(
    r: &mut Resolver<'_>,
    operator: &str,
    left: Option<&Type>,
    right: &Type,
    location: i32,
) -> Option<crate::OperatorInfo> {
    let selection = select_operator(
        r.catalog,
        None,
        operator,
        OperatorKind::Infix,
        left,
        right,
        r.search_path,
    );
    let failure = match selection {
        Selection::NoMatch => crate::resolve::MatchFailure::NoMatch,
        Selection::Ambiguous => crate::resolve::MatchFailure::Ambiguous,
        Selection::Match(operator_result) => return Some(operator_result.operator),
        Selection::Unknown => return None,
    };
    let span = r.operator_span(location);
    r.report_with_span(
        crate::resolve::FindingKind::OperatorMismatch {
            operator: operator.to_owned(),
            left: left.cloned(),
            right: right.clone(),
            failure,
        },
        span,
    );
    None
}

/// The polymorphic types that an operand of an unknown literal resolves to the type of the other
/// operand, like `anyenum` in the comparison operators of enums.
const POLYMORPHIC_SCALAR_TYPES: &[&str] = &[
    "anyelement",
    "anynonarray",
    "anyenum",
    "anycompatible",
    "anycompatiblenonarray",
];

/// Records a string literal operand of `operator` that is coerced to the enum type of the other
/// operand. Postgres first assumes that an unknown literal has the type of the other operand
/// ([`binary_oper_exact`]), and the polymorphic operators of enums resolve it to that type
/// ([`enforce_generic_type_consistency`]).
///
/// [`binary_oper_exact`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L262
/// [`enforce_generic_type_consistency`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2109
fn record_enum_operands(
    r: &mut Resolver<'_>,
    operator: &crate::OperatorInfo,
    left: (Option<&NodeEnum>, &Type),
    right: (Option<&NodeEnum>, &Type),
) {
    if !r.collects_references() {
        return;
    }
    match (left.1, right.1) {
        (Type::UnknownLiteral, Type::Named(other))
            if takes_type_of_other(r, operator.left.as_ref(), other) =>
        {
            r.record_enum_literal(left.0, other);
        }
        (Type::Named(other), Type::UnknownLiteral)
            if takes_type_of_other(r, operator.right.as_ref(), other) =>
        {
            r.record_enum_literal(right.0, other);
        }
        _ => {}
    }
}

/// Whether an operand declared as `declared` takes the type `other` of the other operand.
fn takes_type_of_other(r: &Resolver<'_>, declared: Option<&TypeId>, other: &TypeId) -> bool {
    let Some(declared) = declared else {
        return false;
    };
    declared == other
        || r.catalog.type_by_id(declared).found().is_some_and(|info| {
            info.schema == "pg_catalog" && POLYMORPHIC_SCALAR_TYPES.contains(&info.name.as_str())
        })
}
