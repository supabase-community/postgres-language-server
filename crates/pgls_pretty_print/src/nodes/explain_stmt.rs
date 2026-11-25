use pgls_query::protobuf::ExplainStmt;

use super::prelude::*;

pub(super) fn emit_explain_stmt(e: &mut EventEmitter, n: &ExplainStmt) {
    e.group_start();

    e.token(TokenKind::EXPLAIN_KW);

    // Options (ANALYZE, VERBOSE, COSTS, etc.)
    if !n.options.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.options, emit_utility_option);
    }

    // The query to explain
    if let Some(ref query) = n.query {
        e.line(LineType::SoftOrSpace);
        emit_node(e, query);
    }

    e.group_end();
}
