use pgls_query::protobuf::MultiAssignRef;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_multi_assign_ref(e: &mut EventEmitter, n: &MultiAssignRef) {
    e.group_start(GroupKind::MultiAssignRef);

    if let Some(ref source) = n.source {
        super::emit_node(source, e);
    }

    e.group_end();
}
