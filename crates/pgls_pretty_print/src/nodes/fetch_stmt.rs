use pgls_query::protobuf::FetchStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
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
    // FetchDirection enum values (from protobuf.rs):
    // 0: Undefined
    // 1: FetchForward
    // 2: FetchBackward
    // 3: FetchAbsolute
    // 4: FetchRelative

    // Note: PostgreSQL uses LLONG_MAX (9223372036854775807) to represent "ALL"
    // On some platforms/versions, INT_MAX (2147483647) may be used instead
    let is_all = n.how_many == 0 || n.how_many == 9223372036854775807 || n.how_many == 2147483647;

    // Emit direction and count
    match n.direction {
        1 => {
            // FetchForward - emit explicit FORWARD for ALL, or NEXT for single row
            if is_all {
                e.space();
                e.token(TokenKind::IDENT("FORWARD".to_string()));
            } else if n.how_many == 1 {
                e.space();
                e.token(TokenKind::NEXT_KW);
            } else if n.how_many > 1 {
                e.space();
                e.token(TokenKind::IDENT("FORWARD".to_string()));
            }
        }
        2 => {
            // FetchBackward
            if n.how_many == 1 {
                e.space();
                e.token(TokenKind::IDENT("PRIOR".to_string()));
            } else {
                e.space();
                e.token(TokenKind::IDENT("BACKWARD".to_string()));
            }
        }
        3 => {
            // FetchAbsolute
            e.space();
            e.token(TokenKind::IDENT("ABSOLUTE".to_string()));
        }
        4 => {
            // FetchRelative
            e.space();
            e.token(TokenKind::IDENT("RELATIVE".to_string()));
        }
        _ => {
            // Undefined - should not normally happen
        }
    }

    // Emit count
    // For ABSOLUTE and RELATIVE, the count is always required
    // For FORWARD/BACKWARD, count is only needed when > 1 (or for ALL)
    let needs_count = match n.direction {
        3 | 4 => true, // FetchAbsolute, FetchRelative always need count
        _ => n.how_many > 1 && !is_all,
    };

    if is_all {
        e.space();
        e.token(TokenKind::ALL_KW);
    } else if needs_count {
        e.space();
        e.token(TokenKind::IDENT(n.how_many.to_string()));
    }

    // Emit FROM cursor_name
    if !n.portalname.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::IDENT(n.portalname.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
