use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::StatsElem;

pub(super) fn emit_stats_elem(e: &mut EventEmitter, n: &StatsElem) {
    e.group_start(GroupKind::StatsElem);

    if let Some(ref expr) = n.expr {
        // Expression-based stats entries require parentheses
        // e.g., CREATE STATISTICS ... ON (a || b) FROM ...
        e.token(TokenKind::L_PAREN);
        super::emit_node(expr, e);
        e.token(TokenKind::R_PAREN);
    } else if !n.name.is_empty() {
        super::emit_identifier(e, &n.name);
    }

    e.group_end();
}
