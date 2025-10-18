use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::DeleteStmt;

pub(super) fn emit_delete_stmt(e: &mut EventEmitter, n: &DeleteStmt) {
    emit_delete_stmt_impl(e, n, true);
}

pub(super) fn emit_delete_stmt_no_semicolon(e: &mut EventEmitter, n: &DeleteStmt) {
    emit_delete_stmt_impl(e, n, false);
}

fn emit_delete_stmt_impl(e: &mut EventEmitter, n: &DeleteStmt, with_semicolon: bool) {
    e.group_start(GroupKind::DeleteStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::DELETE_KW);
    e.space();
    e.token(TokenKind::FROM_KW);
    e.space();

    // Emit table name
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if !n.using_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.using_clause, super::emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_node(where_clause, e);
    }

    if !n.returning_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.returning_list, super::emit_node);
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}
