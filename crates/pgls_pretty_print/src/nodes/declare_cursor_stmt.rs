use pgls_query::protobuf::DeclareCursorStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

// Cursor option flags from PostgreSQL's parsenodes.h
// These are defined as (1 << n) where n starts from 0
const CURSOR_OPT_BINARY: i32 = 1 << 0; // 0x01
const CURSOR_OPT_SCROLL: i32 = 1 << 1; // 0x02
const CURSOR_OPT_NO_SCROLL: i32 = 1 << 2; // 0x04
const CURSOR_OPT_INSENSITIVE: i32 = 1 << 3; // 0x08
const CURSOR_OPT_ASENSITIVE: i32 = 1 << 4; // 0x10
const CURSOR_OPT_HOLD: i32 = 1 << 5; // 0x20

pub(super) fn emit_declare_cursor_stmt(e: &mut EventEmitter, n: &DeclareCursorStmt) {
    e.group_start(GroupKind::DeclareCursorStmt);

    e.token(TokenKind::DECLARE_KW);

    // Cursor name
    if !n.portalname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.portalname);
    }

    // Cursor options
    if n.options & CURSOR_OPT_BINARY != 0 {
        e.space();
        e.token(TokenKind::BINARY_KW);
    }
    if n.options & CURSOR_OPT_INSENSITIVE != 0 {
        e.space();
        e.token(TokenKind::INSENSITIVE_KW);
    }
    if n.options & CURSOR_OPT_ASENSITIVE != 0 {
        e.space();
        e.token(TokenKind::ASENSITIVE_KW);
    }
    if n.options & CURSOR_OPT_SCROLL != 0 {
        e.space();
        e.token(TokenKind::SCROLL_KW);
    }
    if n.options & CURSOR_OPT_NO_SCROLL != 0 {
        e.space();
        e.token(TokenKind::NO_KW);
        e.space();
        e.token(TokenKind::SCROLL_KW);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::CURSOR_KW);

    // WITH HOLD / WITHOUT HOLD
    if n.options & CURSOR_OPT_HOLD != 0 {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::HOLD_KW);
    }

    // FOR query
    if let Some(ref query) = n.query {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        super::emit_node(query, e);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
