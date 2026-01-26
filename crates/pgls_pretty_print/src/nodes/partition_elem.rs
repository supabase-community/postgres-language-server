use pgls_query::protobuf::PartitionElem;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_partition_elem(e: &mut EventEmitter, n: &PartitionElem) {
    e.group_start(GroupKind::PartitionElem);

    // Emit column name if present
    if !n.name.is_empty() {
        e.token(TokenKind::IDENT(n.name.clone()));
    } else if let Some(ref expr) = n.expr {
        // Emit expression wrapped in parentheses (required by PostgreSQL syntax)
        e.token(TokenKind::L_PAREN);
        super::emit_node(expr, e);
        e.token(TokenKind::R_PAREN);
    }

    // Emit COLLATE clause if present
    if !n.collation.is_empty() {
        e.space();
        e.token(TokenKind::COLLATE_KW);
        e.space();
        emit_dot_separated_list(e, &n.collation);
    }

    // Emit operator class if present
    if !n.opclass.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.opclass);
    }

    e.group_end();
}
