use pgls_query::protobuf::DiscardStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_discard_stmt(e: &mut EventEmitter, n: &DiscardStmt) {
    e.group_start(GroupKind::DiscardStmt);

    e.token(TokenKind::DISCARD_KW);
    e.space();

    // DiscardMode: Undefined=0, DiscardAll=1, DiscardPlans=2, DiscardSequences=3, DiscardTemp=4
    match n.target {
        1 => e.token(TokenKind::ALL_KW),
        2 => e.token(TokenKind::IDENT("PLANS".to_string())),
        3 => e.token(TokenKind::IDENT("SEQUENCES".to_string())),
        4 => e.token(TokenKind::IDENT("TEMP".to_string())),
        _ => e.token(TokenKind::ALL_KW),
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
