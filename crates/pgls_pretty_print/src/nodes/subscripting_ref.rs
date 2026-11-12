use pgls_query::protobuf::SubscriptingRef;

use super::prelude::*;

pub(super) fn emit_subscripting_ref(e: &mut EventEmitter, n: &SubscriptingRef) {
    e.group_start();

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
        emit_node(e, base);
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
            emit_node(e, lower_node);
        }

        if lower_entry.is_some() {
            e.token(TokenKind::IDENT(":".to_string()));
            if let Some(upper_node) = upper_entry.and_then(|node| node.node.as_ref().map(|_| node))
            {
                emit_node(e, upper_node);
            }
        } else if let Some(upper_node) =
            upper_entry.and_then(|node| node.node.as_ref().map(|_| node))
        {
            emit_node(e, upper_node);
        }

        e.token(TokenKind::R_BRACK);
    }

    e.group_end();
}
