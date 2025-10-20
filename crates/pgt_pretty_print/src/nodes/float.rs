use pgt_query::protobuf::Float;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_float(e: &mut EventEmitter, n: &Float) {
    e.group_start(GroupKind::Float);
    e.token(TokenKind::IDENT(n.fval.clone()));
    e.group_end();
}
