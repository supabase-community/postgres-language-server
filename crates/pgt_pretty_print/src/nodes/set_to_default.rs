use pgt_query::protobuf::SetToDefault;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_set_to_default(e: &mut EventEmitter, _n: &SetToDefault) {
    e.group_start(GroupKind::SetToDefault);
    e.token(TokenKind::DEFAULT_KW);
    e.group_end();
}
