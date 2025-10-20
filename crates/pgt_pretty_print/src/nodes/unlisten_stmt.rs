use pgt_query::protobuf::UnlistenStmt;

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
        e.token(TokenKind::IDENT(n.conditionname.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
