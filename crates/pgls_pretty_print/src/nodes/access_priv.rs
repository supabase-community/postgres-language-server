use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AccessPriv;

use super::node_list::emit_parenthesized_list;

pub(super) fn emit_access_priv(e: &mut EventEmitter, n: &AccessPriv) {
    e.group_start(GroupKind::AccessPriv);

    if !n.priv_name.is_empty() {
        // Privilege names, `alter system` included, follow the keyword case.
        super::emit_keywords(e, &n.priv_name);
    } else {
        // Empty priv_name means ALL privileges
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::PRIVILEGES_KW);
    }

    // Optional column list for column-level privileges
    if !n.cols.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_parenthesized_list(e, &n.cols, super::emit_node);
    }

    e.group_end();
}
