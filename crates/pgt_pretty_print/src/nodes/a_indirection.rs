use pgt_query::protobuf::AIndirection;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_indirection(e: &mut EventEmitter, n: &AIndirection) {
    e.group_start(GroupKind::AIndirection);

    // Emit the base expression
    // Some expressions need parentheses when used with indirection (e.g., ROW(...))
    let needs_parens = if let Some(ref arg) = n.arg {
        matches!(arg.node.as_ref(), Some(pgt_query::NodeEnum::RowExpr(_)))
    } else {
        false
    };

    if needs_parens {
        e.token(TokenKind::L_PAREN);
    }

    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    if needs_parens {
        e.token(TokenKind::R_PAREN);
    }

    // Emit indirection operators (array subscripts, field selections)
    for indirection in &n.indirection {
        // Field selection needs a dot before the field name
        if let Some(pgt_query::NodeEnum::String(_)) = &indirection.node {
            e.token(TokenKind::DOT);
        }
        super::emit_node(indirection, e);
    }

    e.group_end();
}
