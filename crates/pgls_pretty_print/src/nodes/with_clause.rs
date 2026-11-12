use pgls_query::protobuf::WithClause;

use super::prelude::*;

pub(super) fn emit_with_clause(e: &mut EventEmitter, n: &WithClause) {
    // The statements that own a WITH clause emit it directly rather than through emit_node, so this
    // is the only place that can emit the comments written before the WITH keyword.
    emit_with_comments_at(e, n.location, |e| {
        e.group_start();

        e.token(TokenKind::WITH_KW);

        if n.recursive {
            e.space();
            e.token(TokenKind::RECURSIVE_KW);
        }

        if !n.ctes.is_empty() {
            // The first CTE stays on the WITH line. Its body breaks inside its own parentheses.
            e.space();
            emit_comma_separated_list(e, &n.ctes, |e, node| {
                emit_node(e, node);
            });
        }

        e.group_end();
    });
}
