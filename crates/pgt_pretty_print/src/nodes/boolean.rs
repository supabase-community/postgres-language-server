use pgt_query::protobuf::Boolean;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_boolean(e: &mut EventEmitter, n: &Boolean) {
    e.group_start(GroupKind::Boolean);
    e.token(TokenKind::BOOLEAN(n.boolval));
    e.group_end();
}
