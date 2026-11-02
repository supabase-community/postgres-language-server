use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::MergeStmt;

use super::{emit_node, merge_action::emit_merge_when_clause};

pub(super) fn emit_merge_stmt(e: &mut EventEmitter, n: &MergeStmt) {
    emit_merge_stmt_impl(e, n, true);
}

pub(super) fn emit_merge_stmt_no_semicolon(e: &mut EventEmitter, n: &MergeStmt) {
    emit_merge_stmt_impl(e, n, false);
}

fn emit_merge_stmt_impl(e: &mut EventEmitter, n: &MergeStmt, with_semicolon: bool) {
    e.group_start(GroupKind::MergeStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::MERGE_KW);
    e.space();
    e.token(TokenKind::INTO_KW);
    e.space();

    // Target relation
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // USING clause
    if let Some(ref source) = n.source_relation {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_node(source, e);
    }

    // ON condition
    if let Some(ref condition) = n.join_condition {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        emit_node(condition, e);
    }

    // WHEN clauses
    for when_clause_node in &n.merge_when_clauses {
        let when_clause = assert_node_variant!(MergeWhenClause, when_clause_node);
        e.line(LineType::SoftOrSpace);
        emit_merge_when_clause(e, when_clause);
    }

    // RETURNING clause - wrap in group for compact formatting
    if let Some(returning) = &n.returning_clause {
        e.line(LineType::SoftOrSpace);
        super::returning_clause::emit_returning_keyword(e, returning);
        e.indent_start();
        super::node_list::emit_comma_separated_list(e, &returning.exprs, super::emit_node);
        e.indent_end();
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}
