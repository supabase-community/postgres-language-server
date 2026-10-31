use pgls_query::protobuf::Boolean;

use super::prelude::*;

pub(super) fn emit_boolean(e: &mut EventEmitter, n: &Boolean) {
    e.group_start();
    e.token(TokenKind::BOOLEAN(n.boolval));
    e.group_end();
}
