use pgls_query::protobuf::CteSearchClause;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_ctesearch_clause(e: &mut EventEmitter, n: &CteSearchClause) {
    e.group_start(GroupKind::CtesearchClause);

    e.token(TokenKind::SEARCH_KW);
    e.space();
    if n.search_breadth_first {
        e.token(TokenKind::IDENT("BREADTH".to_string()));
    } else {
        e.token(TokenKind::IDENT("DEPTH".to_string()));
    }
    e.space();
    e.token(TokenKind::FIRST_KW);

    if !n.search_col_list.is_empty() {
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.search_col_list, super::emit_node);
    }

    if !n.search_seq_column.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.search_seq_column);
    }

    e.group_end();
}
