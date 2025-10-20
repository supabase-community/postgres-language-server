use pgt_query::protobuf::ImportForeignSchemaStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_import_foreign_schema_stmt(e: &mut EventEmitter, n: &ImportForeignSchemaStmt) {
    e.group_start(GroupKind::ImportForeignSchemaStmt);

    e.token(TokenKind::IMPORT_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::SCHEMA_KW);

    if !n.remote_schema.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.remote_schema.clone()));
    }

    // LIMIT TO / EXCEPT
    if !n.table_list.is_empty() {
        e.space();
        if n.list_type == 1 {
            // LIMIT TO
            e.token(TokenKind::IDENT("LIMIT".to_string()));
            e.space();
            e.token(TokenKind::TO_KW);
        } else {
            // EXCEPT
            e.token(TokenKind::EXCEPT_KW);
        }
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.table_list, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // FROM SERVER
    if !n.server_name.is_empty() {
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(n.server_name.clone()));
    }

    // INTO schema
    if !n.local_schema.is_empty() {
        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();
        e.token(TokenKind::IDENT(n.local_schema.clone()));
    }

    // OPTIONS
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("OPTIONS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
