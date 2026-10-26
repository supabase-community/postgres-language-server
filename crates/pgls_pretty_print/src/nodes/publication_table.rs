use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::PublicationTable;

pub(super) fn emit_publication_table(e: &mut EventEmitter, n: &PublicationTable) {
    e.group_start(GroupKind::PublicationTable);

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if !n.columns.is_empty() {
        e.space();
        super::node_list::emit_parenthesized_list(e, &n.columns, super::emit_node);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_parenthesized_condition(e, GroupKind::PublicationTable, where_clause);
    }

    e.group_end();
}
