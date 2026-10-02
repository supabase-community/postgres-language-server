//! Conservative expression type inference used while resolving query output columns.
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
        CallArg, CoercionContext, Decision, Selection, Type, can_coerce, normalize_type_name,
        select_common_type, select_function, select_operator,
    },
};
pub(super) use assignment::check_assignment;
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
fn common_types(r: &Resolver<'_>, ts: &[Type]) -> Option<Type> {
    match select_common_type(r.catalog, ts) {
        Selection::Match(t) => Some(t),
        _ => None,
    }
}

/// Port of parse_oper.c's `oper`/`make_op` selection; report only proven selection failures.
fn check_operator(
    r: &mut Resolver<'_>,
    operator: &str,
    left: Option<&Type>,
    right: &Type,
    location: i32,
) {
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
        Selection::NoMatch => Some(crate::resolve::MatchFailure::NoMatch),
        Selection::Ambiguous => Some(crate::resolve::MatchFailure::Ambiguous),
        Selection::Match(operator_result) => {
            let Lookup::Found(boolean_info) =
                r.catalog.type_(Some("pg_catalog"), "bool", r.search_path)
            else {
                return;
            };
            let Some(boolean) = boolean_info.id else {
                return;
            };
            if operator_result.result.as_ref() != Some(&Type::Named(boolean)) {
                return;
            }
            None
        }
        Selection::Unknown => None,
    };
    if let Some(failure) = failure {
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
    }
}
