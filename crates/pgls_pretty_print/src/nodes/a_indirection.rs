use pgls_query::protobuf::AIndirection;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_indirection(e: &mut EventEmitter, n: &AIndirection) {
    e.group_start(GroupKind::AIndirection);

    // Emit the base expression
    // Some expressions need parentheses when used with indirection (e.g., ROW(...))
    let needs_parens = if let Some(ref arg) = n.arg {
        let has_indices = n
            .indirection
            .iter()
            .any(|node| matches!(node.node.as_ref(), Some(pgls_query::NodeEnum::AIndices(_))));

        let safe_without_parens = matches!(
            arg.node.as_ref(),
            Some(
                pgls_query::NodeEnum::ColumnRef(_)
                    | pgls_query::NodeEnum::ParamRef(_)
                    | pgls_query::NodeEnum::AIndirection(_)
            )
        );

        matches!(arg.node.as_ref(), Some(pgls_query::NodeEnum::RowExpr(_)))
            || (has_indices && !safe_without_parens)
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
        if let Some(pgls_query::NodeEnum::String(_)) = &indirection.node {
            e.token(TokenKind::DOT);
        }
        super::emit_node(indirection, e);
    }

    e.group_end();
}
