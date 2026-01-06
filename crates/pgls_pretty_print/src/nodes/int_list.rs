use pgls_query::protobuf::IntList;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_int_list(e: &mut EventEmitter, n: &IntList) {
    e.group_start(GroupKind::IntList);

    e.token(TokenKind::L_PAREN);
    super::node_list::emit_comma_separated_list(e, &n.items, super::emit_node);
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
