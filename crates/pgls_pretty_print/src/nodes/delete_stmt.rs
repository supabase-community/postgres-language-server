use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::DeleteStmt;

pub(super) fn emit_delete_stmt(e: &mut EventEmitter, n: &DeleteStmt) {
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
        super::emit_clause_condition(e, where_clause);
    }

    if let Some(returning) = &n.returning_clause {
        e.line(LineType::SoftOrSpace);
        super::emit_returning_clause(e, returning);
    }

    e.group_end();
}
