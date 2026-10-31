use pgls_query::protobuf::PublicationTable;

use super::prelude::*;

pub(super) fn emit_publication_table(e: &mut EventEmitter, n: &PublicationTable) {
    e.group_start();

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if !n.columns.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.columns, emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        e.space();
        emit_parenthesized_condition(e, where_clause);
    }

    e.group_end();
}
