use pgls_query::protobuf::MultiAssignRef;

use super::prelude::*;

pub(super) fn emit_multi_assign_ref(e: &mut EventEmitter, n: &MultiAssignRef) {
    e.group_start();

    if let Some(ref source) = n.source {
        emit_node(e, source);
    }

    e.group_end();
}
