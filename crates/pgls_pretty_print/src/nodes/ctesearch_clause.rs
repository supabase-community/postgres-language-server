use pgls_query::protobuf::CteSearchClause;

use super::prelude::*;

pub(super) fn emit_ctesearch_clause(e: &mut EventEmitter, n: &CteSearchClause) {
    e.group_start(GroupKind::CtesearchClause);

    e.token(TokenKind::SEARCH_KW);
    e.space();
    if n.search_breadth_first {
        e.token(TokenKind::BREADTH_KW);
    } else {
        e.token(TokenKind::DEPTH_KW);
    }
    e.space();
    e.token(TokenKind::FIRST_KW);

    if !n.search_col_list.is_empty() {
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.search_col_list, emit_node);
    }

    if !n.search_seq_column.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.search_seq_column);
    }

    e.group_end();
}
