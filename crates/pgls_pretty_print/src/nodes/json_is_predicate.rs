use pgls_query::protobuf::{JsonIsPredicate, JsonValueType};

use super::prelude::*;

pub(super) fn emit_json_is_predicate(e: &mut EventEmitter, n: &JsonIsPredicate) {
    e.group_start();

    if let Some(ref expr) = n.expr {
        emit_node(e, expr);
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();

    match n.item_type() {
        JsonValueType::Undefined | JsonValueType::JsTypeAny => e.token(TokenKind::JSON_KW),
        JsonValueType::JsTypeObject => {
            e.token(TokenKind::JSON_KW);
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        JsonValueType::JsTypeArray => {
            e.token(TokenKind::JSON_KW);
            e.space();
            e.token(TokenKind::ARRAY_KW);
        }
        JsonValueType::JsTypeScalar => {
            e.token(TokenKind::JSON_KW);
            e.space();
            e.token(TokenKind::SCALAR_KW);
        }
    }

    e.group_end();
}
