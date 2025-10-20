use pgt_query::protobuf::FetchStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_fetch_stmt(e: &mut EventEmitter, n: &FetchStmt) {
    e.group_start(GroupKind::FetchStmt);

    // FETCH or MOVE
    if n.ismove {
        e.token(TokenKind::MOVE_KW);
    } else {
        e.token(TokenKind::FETCH_KW);
    }

    // Direction: NEXT, PRIOR, FIRST, LAST, ABSOLUTE n, RELATIVE n, FORWARD, BACKWARD, etc.
    // FetchDirection enum values:
    // 0: FETCH_FORWARD (default)
    // 1: FETCH_BACKWARD
    // 2: FETCH_ABSOLUTE
    // 3: FETCH_RELATIVE
    // 4: FETCH_FIRST (not documented)
    // 5: FETCH_LAST (not documented)

    // Emit direction and count
    // direction: 0=FORWARD, 1=BACKWARD, 2=ABSOLUTE, 3=RELATIVE
    match n.direction {
        1 => {
            // BACKWARD
            e.space();
            e.token(TokenKind::IDENT("BACKWARD".to_string()));
        }
        2 => {
            // ABSOLUTE
            e.space();
            e.token(TokenKind::IDENT("ABSOLUTE".to_string()));
        }
        3 => {
            // RELATIVE
            e.space();
            e.token(TokenKind::IDENT("RELATIVE".to_string()));
        }
        _ => {
            // FORWARD (default, usually omitted unless explicit)
        }
    }

    // Emit count
    // Note: PostgreSQL uses LLONG_MAX (9223372036854775807) to represent "ALL"
    if n.how_many == 0 || n.how_many == 9223372036854775807 {
        // ALL case (represented as 0 or LLONG_MAX in the AST)
        e.space();
        e.token(TokenKind::ALL_KW);
    } else if n.how_many > 0 {
        e.space();
        e.token(TokenKind::IDENT(n.how_many.to_string()));
    }

    // Emit FROM/IN cursor_name
    if !n.portalname.is_empty() {
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::IDENT(n.portalname.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
