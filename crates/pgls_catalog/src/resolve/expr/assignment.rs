use pgls_query::NodeEnum;

use super::super::resolver::Resolver;
use crate::typing::{CoercionContext, Decision, Type, can_coerce};

/// Reports a value that can't be assigned to a column of INSERT or UPDATE, like
/// [`transformAssignedExpr`] does with an assignment coercion. Unknown literals are coerced at
/// runtime and never reported. `expr` is the value, if it is in the statement itself rather than
/// the output of a query.
///
/// [`transformAssignedExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L455
pub(crate) fn check_assignment(
    r: &mut Resolver<'_>,
    column: &str,
    column_type: Type,
    expr: Option<&NodeEnum>,
    expr_type: Option<Type>,
    location: i32,
) {
    if let Type::Named(target) = &column_type {
        r.record_enum_literal(expr, target);
    }
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
        let span = r.expression_span(location);
        r.report_with_span(
            crate::resolve::FindingKind::AssignmentMismatch {
                column: column.to_owned(),
                column_type,
                expr_type,
            },
            span,
        );
    }
}
