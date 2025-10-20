use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CreateForeignTableStmt;

pub(super) fn emit_create_foreign_table_stmt(e: &mut EventEmitter, n: &CreateForeignTableStmt) {
    e.group_start(GroupKind::CreateForeignTableStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::TABLE_KW);
    e.space();

    // Emit the base CREATE TABLE structure
    if let Some(ref base) = n.base_stmt {
        // Emit table name
        if let Some(ref relation) = base.relation {
            super::emit_range_var(e, relation);
        }

        // Emit column definitions
        if !base.table_elts.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            e.indent_start();
            e.line(crate::emitter::LineType::SoftOrSpace);
            emit_comma_separated_list(e, &base.table_elts, super::emit_node);
            e.indent_end();
            e.line(crate::emitter::LineType::SoftOrSpace);
            e.token(TokenKind::R_PAREN);
        }
    }

    // SERVER clause
    e.space();
    e.token(TokenKind::IDENT("SERVER".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.servername.clone()));

    // OPTIONS clause
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("OPTIONS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
