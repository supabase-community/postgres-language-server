use pgls_query::NodeEnum;
use pgls_query::protobuf::SubscriptingRef;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_subscripting_ref(e: &mut EventEmitter, n: &SubscriptingRef) {
    e.group_start(GroupKind::SubscriptingRef);

    let needs_parens = match n.refexpr.as_ref().and_then(|node| node.node.as_ref()) {
        Some(NodeEnum::ColumnRef(_) | NodeEnum::ParamRef(_) | NodeEnum::SubscriptingRef(_)) => {
            false
        }
        Some(NodeEnum::RowExpr(_)) => true,
        Some(_) => true,
        None => false,
    };

    if needs_parens {
        e.token(TokenKind::L_PAREN);
    }

    if let Some(ref base) = n.refexpr {
        super::emit_node(base, e);
    }

    if needs_parens {
        e.token(TokenKind::R_PAREN);
    }

    let dims = std::cmp::max(n.refupperindexpr.len(), n.reflowerindexpr.len());
    for i in 0..dims {
        e.token(TokenKind::L_BRACK);

        let lower_entry = n.reflowerindexpr.get(i);
        let upper_entry = n.refupperindexpr.get(i);

        if let Some(lower_node) = lower_entry.and_then(|node| node.node.as_ref().map(|_| node)) {
            super::emit_node(lower_node, e);
        }

        if lower_entry.is_some() {
            e.token(TokenKind::IDENT(":".to_string()));
            if let Some(upper_node) = upper_entry.and_then(|node| node.node.as_ref().map(|_| node))
            {
                super::emit_node(upper_node, e);
            }
        } else if let Some(upper_node) =
            upper_entry.and_then(|node| node.node.as_ref().map(|_| node))
        {
            super::emit_node(upper_node, e);
        }

        e.token(TokenKind::R_BRACK);
    }

    e.group_end();
}
