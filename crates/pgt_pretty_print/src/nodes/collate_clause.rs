use pgt_query::protobuf::CollateClause;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_collate_clause(e: &mut EventEmitter, n: &CollateClause) {
    e.group_start(GroupKind::CollateClause);

    // Emit the argument being collated
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
        e.space();
    }

    e.token(TokenKind::COLLATE_KW);
    e.space();

    // Emit the collation name (qualified name)
    // Must quote to preserve case (PostgreSQL lowercases unquoted identifiers)
    for (i, node) in n.collname.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        // Use emit_string_identifier to add quotes
        if let Some(pgt_query::NodeEnum::String(s)) = &node.node {
            super::emit_string_identifier(e, s);
        }
    }

    e.group_end();
}
