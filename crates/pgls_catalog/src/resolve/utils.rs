//! Helpers shared by the modules of the resolver.

use pgls_query::{Node, NodeEnum};

use super::{FunctionParam, expr::infer_expr, resolver::Resolver};
use crate::{
    OperatorInfo, OperatorKind, TypeKind,
    lookup::Lookup,
    typing::{Selection, Type, TypeId, TypedColumn, select_common_type, select_operator},
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

/// The type of a parameter in the body of a SQL function. Postgres parses the body when the
/// function is called, with polymorphic parameters (`anyelement`, `anyarray`, ...) resolved to
/// the types of the call's arguments ([`prepare_sql_fn_parse_info`]), so their type in the body is
/// unknown. Other pseudo-types are not allowed for parameters of SQL functions
/// ([`fmgr_sql_validator`]).
///
/// [`prepare_sql_fn_parse_info`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/executor/functions.c#L286
/// [`fmgr_sql_validator`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/pg_proc.c#L873
pub(super) fn function_param_type(r: &Resolver<'_>, param: &FunctionParam) -> Option<Type> {
    let Lookup::Found(mut info) = r.catalog.type_(
        param.type_schema.as_deref(),
        &param.type_name,
        r.search_path,
    ) else {
        return None;
    };
    if info.kind == Some(TypeKind::Pseudo) {
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
/// [`make_op`] (through [`oper`]) does. Reports only proven selection failures. Returns the
/// selected operator.
///
/// [`make_op`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L660
/// [`oper`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L370
pub(super) fn check_operator(
    r: &mut Resolver<'_>,
    operator: &str,
    left: Option<&Type>,
    right: &Type,
    location: i32,
) -> Option<OperatorInfo> {
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
pub(super) fn record_enum_operands(
    r: &mut Resolver<'_>,
    operator: &OperatorInfo,
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

/// Records the string literals among `values` that are coerced to `common`, the common type of
/// the values of `CASE` or `COALESCE`, if it is an enum ([`coerce_to_common_type`]).
///
/// [`coerce_to_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1592
pub(super) fn record_enum_values<'a>(
    r: &mut Resolver<'_>,
    values: impl IntoIterator<Item = Option<&'a NodeEnum>>,
    common: Option<&Type>,
) {
    let Some(Type::Named(common)) = common else {
        return;
    };
    for value in values {
        r.record_enum_literal(value, common);
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
