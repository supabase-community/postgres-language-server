use pgt_query::protobuf::UpdateStmt;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use crate::nodes::res_target::emit_set_clause;

use super::emit_node;
use super::node_list::emit_comma_separated_list;

pub(super) fn emit_update_stmt(e: &mut EventEmitter, n: &UpdateStmt) {
    emit_update_stmt_impl(e, n, true);
}

pub(super) fn emit_update_stmt_no_semicolon(e: &mut EventEmitter, n: &UpdateStmt) {
    emit_update_stmt_impl(e, n, false);
}

fn emit_update_stmt_impl(e: &mut EventEmitter, n: &UpdateStmt, with_semicolon: bool) {
    e.group_start(GroupKind::UpdateStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::UPDATE_KW);
    e.space();

    if let Some(ref range_var) = n.relation {
        super::emit_range_var(e, range_var)
    }

    if !n.target_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        emit_comma_separated_list(e, &n.target_list, |n, e| {
            emit_set_clause(e, assert_node_variant!(ResTarget, n))
        });
    }

    if !n.from_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        emit_comma_separated_list(e, &n.from_clause, super::emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        e.space();
        emit_node(where_clause, e);
    }

    if !n.returning_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        emit_comma_separated_list(e, &n.returning_list, super::emit_node);
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}
