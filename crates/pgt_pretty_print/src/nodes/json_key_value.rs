use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonKeyValue;

use super::json_value_expr::emit_json_value_expr;

pub(super) fn emit_json_key_value(e: &mut EventEmitter, n: &JsonKeyValue) {
    e.group_start(GroupKind::JsonKeyValue);

    if let Some(ref key) = n.key {
        super::emit_node(key, e);
    }

    e.space();
    e.token(TokenKind::IDENT(":".to_string()));
    e.space();

    if let Some(ref value) = n.value {
        emit_json_value_expr(e, value);
    }

    e.group_end();
}
