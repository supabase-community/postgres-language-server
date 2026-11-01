use pgls_query::protobuf::CheckPointStmt;

use super::prelude::*;

pub(super) fn emit_check_point_stmt(e: &mut EventEmitter, _n: &CheckPointStmt) {
    e.group_start(GroupKind::CheckPointStmt);

    e.token(TokenKind::CHECKPOINT_KW);

    e.group_end();
}
