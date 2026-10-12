use pgls_query::protobuf::IndexStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_parenthesized_list;

pub(super) fn emit_index_stmt(e: &mut EventEmitter, n: &IndexStmt) {
    emit_index_stmt_impl(e, n, true);
}

pub(super) fn emit_index_stmt_no_semicolon(e: &mut EventEmitter, n: &IndexStmt) {
    emit_index_stmt_impl(e, n, false);
}

fn emit_index_stmt_impl(e: &mut EventEmitter, n: &IndexStmt, with_semicolon: bool) {
    e.group_start(GroupKind::IndexStmt);

    // The head, up to the opening parenthesis of the columns, stays on one line when it fits. The
    // columns break inside their parentheses, and the clauses after them each start a line when
    // they do not fit.
    e.group_start(GroupKind::IndexStmt);
    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.unique {
        e.token(TokenKind::UNIQUE_KW);
        e.space();
    }

    e.token(TokenKind::INDEX_KW);

    if n.concurrent {
        e.space();
        e.token(TokenKind::CONCURRENTLY_KW);
    }

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    // Index name
    if !n.idxname.is_empty() {
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.idxname);
    }

    e.indent_start();

    // ON table
    if let Some(ref relation) = n.relation {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        super::emit_range_var(e, relation);
    }

    // USING access_method
    if !n.access_method.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.access_method);
    }

    e.indent_end();
    e.group_end();

    // Index columns/expressions
    if !n.index_params.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.index_params, super::emit_node);
    }

    if n.nulls_not_distinct {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::NULLS_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::DISTINCT_KW);
    }

    // INCLUDE columns
    if !n.index_including_params.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::INCLUDE_KW);
        e.space();
        emit_parenthesized_list(e, &n.index_including_params, super::emit_node);
    }

    // WITH options
    if !n.options.is_empty() {
        super::emit_layout_break(e);
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, super::emit_node);
    }

    // TABLESPACE
    if !n.table_space.is_empty() {
        super::emit_layout_break(e);
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.table_space);
    }

    // WHERE clause (partial index), kept on its keyword line when it fits there
    if let Some(ref where_clause) = n.where_clause {
        super::emit_layout_break(e);
        e.group_start(GroupKind::IndexStmt);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, where_clause);
        e.group_end();
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }
    e.group_end();
}
