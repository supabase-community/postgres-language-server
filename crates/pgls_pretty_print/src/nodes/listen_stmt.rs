use pgls_query::protobuf::ListenStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_listen_stmt(e: &mut EventEmitter, n: &ListenStmt) {
    e.group_start(GroupKind::ListenStmt);

    e.token(TokenKind::LISTEN_KW);
    e.space();
    super::emit_identifier_maybe_quoted(e, &n.conditionname);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
