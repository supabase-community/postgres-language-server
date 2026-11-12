use pgls_query::protobuf::JsonKeyValue;

use super::emit_json_value_expr;
use super::prelude::*;

pub(super) fn emit_json_key_value(e: &mut EventEmitter, n: &JsonKeyValue) {
    e.group_start();

    if let Some(ref key) = n.key {
        emit_node(e, key);
    }

    e.space();
    e.token(TokenKind::IDENT(":".to_string()));
    e.space();

    if let Some(ref value) = n.value {
        emit_json_value_expr(e, value);
    }

    e.group_end();
}
