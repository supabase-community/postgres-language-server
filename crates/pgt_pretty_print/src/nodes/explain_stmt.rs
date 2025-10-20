use pgt_query::protobuf::ExplainStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_explain_stmt(e: &mut EventEmitter, n: &ExplainStmt) {
    e.group_start(GroupKind::ExplainStmt);

    e.token(TokenKind::EXPLAIN_KW);

    // Options (ANALYZE, VERBOSE, etc.) - simplified for now
    if !n.options.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // The query to explain
    if let Some(ref query) = n.query {
        e.space();
        super::emit_node(query, e);
    }

    e.group_end();
}
