use pgls_query::protobuf::ParamRef;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_param_ref(e: &mut EventEmitter, n: &ParamRef) {
    e.group_start(GroupKind::ParamRef);

    // Emit $1, $2, etc. - handle overflow as unsigned
    // n.number is i32, but PostgreSQL param numbers are always positive
    // When large numbers overflow, we need to interpret as unsigned
    e.token(TokenKind::IDENT(format!("${}", n.number as u32)));

    e.group_end();
}
