use super::super::resolver::Resolver;
use crate::typing::{CoercionContext, Decision, Type, can_coerce};

pub(crate) fn check_assignment(
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
