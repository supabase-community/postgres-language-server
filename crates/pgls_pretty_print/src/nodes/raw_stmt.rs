use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::RawStmt;

pub(super) fn emit_raw_stmt(e: &mut EventEmitter, n: &RawStmt) {
    e.group_start(GroupKind::RawStmt);

    if let Some(ref stmt) = n.stmt {
        super::emit_node(stmt, e);
    }

    e.group_end();
}
