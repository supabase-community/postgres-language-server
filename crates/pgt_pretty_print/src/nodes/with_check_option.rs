use pgt_query::protobuf::WithCheckOption;

use crate::emitter::{EventEmitter, GroupKind};

/// Emit a WithCheckOption (planner check option node for views)
/// These are internal planner representations
/// We emit the qual expression if available
pub(super) fn emit_with_check_option(e: &mut EventEmitter, n: &WithCheckOption) {
    e.group_start(GroupKind::WithCheckOption);

    // WithCheckOption is the planner's representation of view check options
    // Emit the qual (WHERE condition) if available
    if let Some(qual) = n.qual.as_deref() {
        super::emit_node(qual, e);
    }

    e.group_end();
}
