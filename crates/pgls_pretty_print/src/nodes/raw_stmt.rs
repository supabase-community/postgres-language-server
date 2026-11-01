use pgls_query::protobuf::RawStmt;

use super::prelude::*;

pub(super) fn emit_raw_stmt(e: &mut EventEmitter, n: &RawStmt) {
    e.group_start();

    if let Some(ref stmt) = n.stmt {
        emit_node(e, stmt);
    }

    e.group_end();
}
