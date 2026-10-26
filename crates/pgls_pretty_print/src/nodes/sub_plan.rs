use pgls_query::protobuf::SubPlan;

use super::prelude::*;

/// Emit a SubPlan (planner subquery node)
/// These are internal planner representations
/// We try to emit the test expression or fallback to a placeholder
pub(super) fn emit_sub_plan(e: &mut EventEmitter, n: &SubPlan) {
    e.group_start(GroupKind::SubPlan);

    // SubPlan is the planner's representation of subqueries
    // Emit the test expression if available, otherwise a placeholder
    if let Some(testexpr) = n.testexpr.as_deref() {
        emit_node(testexpr, e);
    } else if let Some(first_arg) = n.args.first() {
        emit_node(first_arg, e);
    } else if !n.plan_name.is_empty() {
        e.token(TokenKind::IDENT(n.plan_name.clone()));
    } else {
        e.token(TokenKind::IDENT(format!("SubPlan{}", n.plan_id)));
    }

    e.group_end();
}
