use pgt_query::protobuf::IndexStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_index_stmt(e: &mut EventEmitter, n: &IndexStmt) {
    e.group_start(GroupKind::IndexStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    // TODO: Handle UNIQUE, CONCURRENTLY flags (not in protobuf?)

    e.token(TokenKind::INDEX_KW);

    // Index name
    if !n.idxname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.idxname);
    }

    // ON table
    if let Some(ref relation) = n.relation {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        super::emit_range_var(e, relation);
    }

    // USING access_method
    if !n.access_method.is_empty() {
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(n.access_method.clone()));
    }

    // Index columns/expressions
    if !n.index_params.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.index_params, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // INCLUDE columns
    if !n.index_including_params.is_empty() {
        e.space();
        e.token(TokenKind::INCLUDE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.index_including_params, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // WITH options
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // TABLESPACE
    if !n.table_space.is_empty() {
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(n.table_space.clone()));
    }

    // WHERE clause (partial index)
    if let Some(ref where_clause) = n.where_clause {
        e.space();
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_node(where_clause, e);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
