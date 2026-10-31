use pgls_query::protobuf::CreateAmStmt;

use super::prelude::*;

pub(super) fn emit_create_am_stmt(e: &mut EventEmitter, n: &CreateAmStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::ACCESS_KW);
    e.space();
    e.token(TokenKind::METHOD_KW);

    if !n.amname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.amname);
    }

    // TYPE
    // amtype is a single character: 'i' = INDEX, 't' = TABLE
    if !n.amtype.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TYPE_KW);
        e.space();
        match n.amtype.as_str() {
            "i" => e.token(TokenKind::INDEX_KW),
            "t" => e.token(TokenKind::TABLE_KW),
            _ => e.token(TokenKind::IDENT(n.amtype.clone())), // fallback to original value
        };
    }

    // HANDLER
    if !n.handler_name.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::HANDLER_KW);
        e.space();
        emit_dot_separated_list(e, &n.handler_name);
    }

    e.group_end();
}
