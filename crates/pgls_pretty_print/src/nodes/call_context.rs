use pgls_query::protobuf::CallContext;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_call_context(e: &mut EventEmitter, _n: &CallContext) {
    e.group_start(GroupKind::CallContext);
    // CallContext nodes are executor metadata; nothing to render for surface SQL.
    e.group_end();
}
