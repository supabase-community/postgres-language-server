use pgt_query::protobuf::DiscardStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_discard_stmt(e: &mut EventEmitter, n: &DiscardStmt) {
    e.group_start(GroupKind::DiscardStmt);

    e.token(TokenKind::DISCARD_KW);
    e.space();

    // DiscardMode: ALL=0, PLANS=1, SEQUENCES=2, TEMP=3
    match n.target {
        0 => e.token(TokenKind::ALL_KW),
        1 => e.token(TokenKind::IDENT("PLANS".to_string())),
        2 => e.token(TokenKind::IDENT("SEQUENCES".to_string())),
        3 => e.token(TokenKind::IDENT("TEMP".to_string())),
        _ => e.token(TokenKind::ALL_KW),
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
