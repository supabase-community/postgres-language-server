use pgls_query::{NodeEnum, protobuf::AIndirection};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_indirection(e: &mut EventEmitter, n: &AIndirection) {
    e.group_start(GroupKind::AIndirection);

    let needs_parens = n
        .arg
        .as_ref()
        .is_some_and(|arg| base_needs_parens(arg.node.as_ref(), n));

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
        // Field selection and star expansion need a dot before them
        match &indirection.node {
            Some(NodeEnum::String(_)) | Some(NodeEnum::AStar(_)) => {
                e.token(TokenKind::DOT);
            }
            _ => {}
        }
        super::emit_node(indirection, e);
    }

    e.group_end();
}

/// The parser only builds an `AIndirection` without parentheses for a parameter or for a column
/// reference whose indirection starts with a subscript: `r.f` and `r.*` stay a `ColumnRef`, and
/// `x[1][2]` is a single `AIndirection` while `(x[1])[2]` nests one in another. Everything else
/// keeps its shape only when the base is parenthesized.
fn base_needs_parens(arg: Option<&NodeEnum>, n: &AIndirection) -> bool {
    match arg {
        Some(NodeEnum::ParamRef(_)) => false,
        Some(NodeEnum::ColumnRef(_)) => !matches!(
            n.indirection.first().and_then(|node| node.node.as_ref()),
            Some(NodeEnum::AIndices(_))
        ),
        // `(SELECT ...)[1]` already carries its parentheses
        Some(NodeEnum::SubLink(sub_link)) => {
            sub_link.sub_link_type() != pgls_query::protobuf::SubLinkType::ExprSublink
        }
        Some(_) => true,
        None => false,
    }
}
