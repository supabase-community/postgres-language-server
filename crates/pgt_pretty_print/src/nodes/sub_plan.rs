use pgt_query::protobuf::{AlternativeSubPlan, SubPlan};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

/// Emit a SubPlan (planner subquery node)
/// These are internal planner representations
/// We try to emit the test expression or fallback to a placeholder
pub(super) fn emit_sub_plan(e: &mut EventEmitter, n: &SubPlan) {
    e.group_start(GroupKind::SubPlan);

    // SubPlan is the planner's representation of subqueries
    // Emit the test expression if available, otherwise a placeholder
    if let Some(testexpr) = n.testexpr.as_deref() {
        super::emit_node(testexpr, e);
    } else if let Some(first_arg) = n.args.first() {
        super::emit_node(first_arg, e);
    } else if !n.plan_name.is_empty() {
        e.token(TokenKind::IDENT(n.plan_name.clone()));
    } else {
        e.token(TokenKind::IDENT(format!("SubPlan{}", n.plan_id)));
    }

    e.group_end();
}

/// Emit an AlternativeSubPlan (planner alternative subplan node)
/// These represent multiple subplan options for the planner
/// We emit the first available subplan
pub(super) fn emit_alternative_sub_plan(e: &mut EventEmitter, n: &AlternativeSubPlan) {
    e.group_start(GroupKind::AlternativeSubPlan);

    // AlternativeSubPlan contains multiple subplan choices
    // Emit the first one if available
    if let Some(first) = n.subplans.first() {
        if let Some(inner) = first.node.as_ref() {
            super::emit_node_enum(inner, e);
        }
    }

    e.group_end();
}
