use pgls_query::protobuf::Integer;

use super::prelude::*;

pub(super) fn emit_integer(e: &mut EventEmitter, n: &Integer) {
    e.group_start();
    e.token(TokenKind::IDENT(n.ival.to_string()));
    e.group_end();
}
