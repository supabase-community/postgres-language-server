use pgls_query::protobuf::OidList;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_oid_list(e: &mut EventEmitter, n: &OidList) {
    e.group_start(GroupKind::OidList);

    e.token(TokenKind::L_PAREN);
    super::node_list::emit_comma_separated_list(e, &n.items, super::emit_node);
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
