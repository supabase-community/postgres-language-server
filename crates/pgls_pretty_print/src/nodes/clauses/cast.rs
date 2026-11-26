//! The name of a cast in DDL: `(source AS target)`.

use crate::nodes::prelude::*;

/// Emits a cast as DDL names it, `(source AS target)`. The parser stores it as `[source, target]`.
pub(crate) fn emit_cast_object(e: &mut EventEmitter, node: &Node) {
    if let Some(NodeEnum::List(list)) = node.node.as_ref()
        && let [source, target] = list.items.as_slice()
    {
        e.token(TokenKind::L_PAREN);
        emit_node(e, source);
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_node(e, target);
        e.token(TokenKind::R_PAREN);
        return;
    }

    // Fallback for unexpected structure
    emit_node(e, node);
}
