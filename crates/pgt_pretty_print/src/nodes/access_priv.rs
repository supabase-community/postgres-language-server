use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AccessPriv;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_access_priv(e: &mut EventEmitter, n: &AccessPriv) {
    e.group_start(GroupKind::AccessPriv);

    if !n.priv_name.is_empty() {
        e.token(TokenKind::IDENT(n.priv_name.clone().to_uppercase()));
    } else {
        // Empty priv_name means ALL privileges
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::IDENT("PRIVILEGES".to_string()));
    }

    // Optional column list for column-level privileges
    if !n.cols.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.cols, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
