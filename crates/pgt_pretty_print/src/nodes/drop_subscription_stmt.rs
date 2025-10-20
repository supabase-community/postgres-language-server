use pgt_query::protobuf::DropSubscriptionStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_drop_subscription_stmt(e: &mut EventEmitter, n: &DropSubscriptionStmt) {
    e.group_start(GroupKind::DropSubscriptionStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::SUBSCRIPTION_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.subname.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.subname.clone()));
    }

    // Add CASCADE or RESTRICT if specified
    if n.behavior != 0 {
        e.space();
        match n.behavior {
            1 => e.token(TokenKind::CASCADE_KW),
            _ => e.token(TokenKind::RESTRICT_KW),
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
