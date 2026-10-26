use pgls_query::protobuf::Float;

use super::prelude::*;

pub(super) fn emit_float(e: &mut EventEmitter, n: &Float) {
    e.group_start();
    e.token(TokenKind::IDENT(n.fval.clone()));
    e.group_end();
}
