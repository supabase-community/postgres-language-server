use pgls_query::protobuf::ListenStmt;

use super::prelude::*;

pub(super) fn emit_listen_stmt(e: &mut EventEmitter, n: &ListenStmt) {
    e.group_start();

    e.token(TokenKind::LISTEN_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.conditionname);

    e.group_end();
}
