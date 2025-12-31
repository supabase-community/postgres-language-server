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
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.columns, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    if let Some(ref where_clause) = n.where_clause {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_clause_condition(e, where_clause);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
