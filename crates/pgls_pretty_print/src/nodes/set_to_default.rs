use pgls_query::protobuf::SetToDefault;

use super::prelude::*;

pub(super) fn emit_set_to_default(e: &mut EventEmitter, _n: &SetToDefault) {
    e.group_start();
    e.token(TokenKind::DEFAULT_KW);
    e.group_end();
}
