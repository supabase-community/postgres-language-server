use pgls_query::protobuf::WithClause;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_with_clause(e: &mut EventEmitter, n: &WithClause) {
    // The statements that own a WITH clause emit it directly rather than through emit_node, so this
    // is the only place that can emit the comments written before the WITH keyword.
    super::emit_with_comments_at(e, n.location, |e| {
        e.group_start(GroupKind::WithClause);

        e.token(TokenKind::WITH_KW);

        if n.recursive {
            e.space();
            e.token(TokenKind::RECURSIVE_KW);
        }

        if !n.ctes.is_empty() {
            // The first CTE stays on the WITH line. Its body breaks inside its own parentheses.
            e.space();
            emit_comma_separated_list(e, &n.ctes, |node, e| {
                super::emit_node(node, e);
            });
        }

        e.group_end();
    });
}
