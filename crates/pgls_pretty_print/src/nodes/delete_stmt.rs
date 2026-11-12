use pgls_query::protobuf::DeleteStmt;

use super::prelude::*;

pub(super) fn emit_delete_stmt(e: &mut EventEmitter, n: &DeleteStmt) {
    e.group_start();

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

    if let Some(first) = n.using_clause.first() {
        emit_comments_before_keyword(e, "using", first);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_comma_separated_list(e, &n.using_clause, emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        emit_comments_before_keyword(e, "where", where_clause);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        emit_clause_condition(e, where_clause);
    }

    if let Some(returning) = &n.returning_clause {
        if let Some(first) = returning.options.first().or(returning.exprs.first()) {
            emit_comments_before_keyword(e, "returning", first);
        }
        e.line(LineType::SoftOrSpace);
        super::emit_returning_clause(e, returning);
    }

    e.group_end();
}
