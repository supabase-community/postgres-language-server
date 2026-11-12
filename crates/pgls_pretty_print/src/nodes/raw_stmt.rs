use pgls_query::protobuf::RawStmt;

use super::prelude::*;

pub(super) fn emit_raw_stmt(e: &mut EventEmitter, n: &RawStmt) {
    e.group_start(GroupKind::RawStmt);

    if let Some(ref stmt) = n.stmt {
        emit_node(stmt, e);
    }

    e.group_end();
}
