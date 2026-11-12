use pgls_query::protobuf::WithCheckOption;

use super::prelude::*;

/// Emit a WithCheckOption (planner check option node for views)
/// These are internal planner representations
/// We emit the qual expression if available
pub(super) fn emit_with_check_option(e: &mut EventEmitter, n: &WithCheckOption) {
    e.group_start();

    // WithCheckOption is the planner's representation of view check options
    // Emit the qual (WHERE condition) if available
    if let Some(qual) = n.qual.as_deref() {
        emit_node(e, qual);
    }

    e.group_end();
}
