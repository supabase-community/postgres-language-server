use pgt_query::protobuf::DeclareCursorStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_declare_cursor_stmt(e: &mut EventEmitter, n: &DeclareCursorStmt) {
    e.group_start(GroupKind::DeclareCursorStmt);

    e.token(TokenKind::DECLARE_KW);

    // Cursor name
    if !n.portalname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.portalname);
    }

    // Cursor options (bitmap flags: BINARY, INSENSITIVE, SCROLL, etc.)
    // TODO: Parse options bitmap and emit appropriate keywords
    // For now, we skip detailed option parsing

    e.space();
    e.token(TokenKind::CURSOR_KW);

    // FOR query
    if let Some(ref query) = n.query {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        super::emit_node(query, e);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
