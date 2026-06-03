use pgls_query::protobuf::UnlistenStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_unlisten_stmt(e: &mut EventEmitter, n: &UnlistenStmt) {
    e.group_start(GroupKind::UnlistenStmt);

    e.token(TokenKind::UNLISTEN_KW);
    e.space();

    if n.conditionname.is_empty() || n.conditionname == "*" {
        e.token(TokenKind::IDENT("*".to_string()));
    } else {
        super::emit_identifier_maybe_quoted(e, &n.conditionname);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
