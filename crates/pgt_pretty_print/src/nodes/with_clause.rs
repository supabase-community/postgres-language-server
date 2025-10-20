use pgt_query::protobuf::WithClause;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_with_clause(e: &mut EventEmitter, n: &WithClause) {
    e.group_start(GroupKind::WithClause);

    e.token(TokenKind::WITH_KW);

    if n.recursive {
        e.space();
        e.token(TokenKind::RECURSIVE_KW);
    }

    if !n.ctes.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.ctes, |node, e| {
            super::emit_node(node, e);
        });
    }

    e.group_end();
}
