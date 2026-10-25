//! Helpers shared by the modules of the resolver.

use pgls_query::{Node, NodeEnum};

use super::{FunctionParam, expr::infer_expr, resolver::Resolver};
use crate::{
    OperatorKind,
    lookup::Lookup,
    typing::{Selection, Type, TypedColumn, select_common_type, select_operator},
};

/// The builtin type `pg_catalog.<name>`.
pub(super) fn named(r: &Resolver<'_>, name: &str) -> Option<Type> {
    r.catalog.builtin_type(name).map(Type::Named)
}

/// The type of a field of a row.
pub(super) fn field_type(r: &Resolver<'_>, row: &Type, field: &str) -> Option<Type> {
    let columns = match row {
        Type::Record(columns) => columns.clone(),
        Type::Named(id) => {
            let Lookup::Found(info) = r.catalog.type_by_id(id) else {
                return None;
            };
            info.attributes?
                .into_iter()
                .map(|attribute| TypedColumn {
                    name: attribute.name,
                    ty: attribute.ty,
                })
                .collect()
        }
        Type::UnknownLiteral => return None,
    };
    columns.into_iter().find(|column| column.name == field)?.ty
}

/// The type of a parameter of the SQL function whose body is being resolved.
pub(super) fn function_param_type(r: &Resolver<'_>, param: &FunctionParam) -> Option<Type> {
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

/// The common type of the arguments, if all their types are known ([`select_common_type`]).
///
/// [`select_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1345
pub(super) fn common(r: &mut Resolver<'_>, args: &[Node]) -> Option<Type> {
    let types = args
        .iter()
        .map(|arg| arg.node.as_ref().and_then(|x| infer_expr(r, x)))
        .collect::<Vec<_>>();
    common_types(r, &types.into_iter().collect::<Option<Vec<_>>>()?)
}
pub(super) fn common_types(r: &Resolver<'_>, ts: &[Type]) -> Option<Type> {
    match select_common_type(r.catalog, ts) {
        Selection::Match(t) => Some(t),
        _ => None,
    }
}

/// Checks the `=` that `IS DISTINCT FROM`, `NULLIF` and simple `CASE` compare with, like
/// [`make_op`] (through [`oper`]) does. Reports only proven selection failures.
///
/// [`make_op`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L660
/// [`oper`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L370
pub(super) fn check_operator(
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
            let Some(boolean) = r.catalog.builtin_type("bool") else {
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

/// The location of an expression.
pub(super) fn node_location(node: &Node) -> Option<i32> {
    Some(match node.node.as_ref()? {
        NodeEnum::AConst(n) => n.location,
        NodeEnum::ColumnRef(n) => n.location,
        NodeEnum::AExpr(n) => n.location,
        NodeEnum::FuncCall(n) => n.location,
        NodeEnum::TypeCast(n) => n.location,
        NodeEnum::ParamRef(n) => n.location,
        NodeEnum::CaseExpr(n) => n.location,
        NodeEnum::CoalesceExpr(n) => n.location,
        NodeEnum::MinMaxExpr(n) => n.location,
        NodeEnum::AArrayExpr(n) => n.location,
        NodeEnum::BoolExpr(n) => n.location,
        NodeEnum::NullTest(n) => n.location,
        NodeEnum::BooleanTest(n) => n.location,
        NodeEnum::SubLink(n) => n.location,
        NodeEnum::RowExpr(n) => n.location,
        NodeEnum::SqlvalueFunction(n) => n.location,
        NodeEnum::CollateClause(n) => n.location,
        NodeEnum::AIndirection(_) => return None,
        _ => return None,
    })
}
