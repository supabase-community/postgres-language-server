use pgt_query::protobuf::Alias;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alias(e: &mut EventEmitter, n: &Alias) {
    e.group_start(GroupKind::Alias);

    if n.aliasname.is_empty() {
        e.group_end();
        return;
    }

    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::IDENT(n.aliasname.clone()));

    // Add column aliases if present
    if !n.colnames.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.colnames, |node, e| {
            // Column names in alias are String nodes
            if let Some(pgt_query::NodeEnum::String(s)) = node.node.as_ref() {
                e.token(TokenKind::IDENT(s.sval.clone()));
            } else {
                super::emit_node(node, e);
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
