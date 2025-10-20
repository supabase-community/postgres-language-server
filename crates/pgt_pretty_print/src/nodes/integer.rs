use pgt_query::protobuf::Integer;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_integer(e: &mut EventEmitter, n: &Integer) {
    e.group_start(GroupKind::Integer);
    e.token(TokenKind::IDENT(n.ival.to_string()));
    e.group_end();
}
