use pgls_query::protobuf::ReturnStmt;

use super::prelude::*;

pub(super) fn emit_return_stmt(e: &mut EventEmitter, n: &ReturnStmt) {
    e.group_start(GroupKind::ReturnStmt);

    e.token(TokenKind::RETURN_KW);

    if let Some(ref value) = n.returnval {
        e.space();
        emit_node(value, e);
    }

    e.group_end();
}
