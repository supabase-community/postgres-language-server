use pgls_query::protobuf::UpdateStmt;

use super::prelude::*;

pub(super) fn emit_update_stmt(e: &mut EventEmitter, n: &UpdateStmt) {
    e.group_start(GroupKind::UpdateStmt);

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);

        if let Some(ref range_var) = n.relation {
            e.take_leading_comments_at(range_var.location);
        }
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
        super::res_target::emit_set_clause_list(e, &n.target_list);
    }

    if !n.from_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        emit_comma_separated_list(e, &n.from_clause, emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        emit_clause_condition(e, where_clause);
    }

    if !n.returning_list.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        emit_comma_separated_list(e, &n.returning_list, emit_node);
    }

    e.group_end();
}
