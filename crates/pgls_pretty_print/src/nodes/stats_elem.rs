use pgls_query::protobuf::StatsElem;

use super::prelude::*;

pub(super) fn emit_stats_elem(e: &mut EventEmitter, n: &StatsElem) {
    e.group_start(GroupKind::StatsElem);

    if let Some(ref expr) = n.expr {
        // Expression-based stats entries require parentheses
        // e.g., CREATE STATISTICS ... ON (a || b) FROM ...
        e.token(TokenKind::L_PAREN);
        emit_node(expr, e);
        e.token(TokenKind::R_PAREN);
    } else if !n.name.is_empty() {
        emit_identifier_maybe_quoted(e, &n.name);
    }

    e.group_end();
}
