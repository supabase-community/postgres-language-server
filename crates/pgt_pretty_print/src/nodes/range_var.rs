use pgt_query::protobuf::RangeVar;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_range_var(e: &mut EventEmitter, n: &RangeVar) {
    e.group_start(GroupKind::RangeVar);

    if !n.schemaname.is_empty() {
        e.token(TokenKind::IDENT(n.schemaname.clone()));
        e.token(TokenKind::DOT);
    }

    e.token(TokenKind::IDENT(n.relname.clone()));

    // Emit alias if present
    if let Some(ref alias) = n.alias {
        e.space();
        super::emit_alias(e, alias);
    }

    e.group_end();
}
