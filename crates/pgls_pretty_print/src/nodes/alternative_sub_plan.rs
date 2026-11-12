use pgls_query::protobuf::AlternativeSubPlan;

use super::prelude::*;

/// Emit an AlternativeSubPlan (planner alternative subplan node)
/// These represent multiple subplan options for the planner
/// We emit the first available subplan
pub(super) fn emit_alternative_sub_plan(e: &mut EventEmitter, n: &AlternativeSubPlan) {
    e.group_start();

    // AlternativeSubPlan contains multiple subplan choices
    // Emit the first one if available
    if let Some(first) = n.subplans.first()
        && let Some(inner) = first.node.as_ref()
    {
        emit_node_enum(e, inner);
    }

    e.group_end();
}
