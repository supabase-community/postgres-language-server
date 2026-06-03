use pgls_query::protobuf::Alias;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alias(e: &mut EventEmitter, n: &Alias) {
    e.group_start(GroupKind::Alias);

    if n.aliasname.is_empty() {
        e.group_end();
        return;
    }

    e.token(TokenKind::AS_KW);
    e.space();
    super::emit_identifier_maybe_quoted(e, &n.aliasname);

    // Add column aliases if present
    if !n.colnames.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.colnames, |node, e| {
            // Column names in alias are String nodes
            if let Some(pgls_query::NodeEnum::String(s)) = node.node.as_ref() {
                super::emit_identifier_maybe_quoted(e, &s.sval);
            } else {
                super::emit_node(node, e);
            }
        });
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
