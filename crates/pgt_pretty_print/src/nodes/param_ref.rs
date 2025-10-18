use pgt_query::protobuf::ParamRef;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_param_ref(e: &mut EventEmitter, n: &ParamRef) {
    e.group_start(GroupKind::ParamRef);

    // Emit $1, $2, etc.
    e.token(TokenKind::IDENT(format!("${}", n.number)));

    e.group_end();
}
