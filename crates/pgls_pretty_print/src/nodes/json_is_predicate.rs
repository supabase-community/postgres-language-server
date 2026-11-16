use pgls_query::protobuf::{JsonIsPredicate, JsonValueType};

use super::prelude::*;

/// `expr IS JSON [VALUE | OBJECT | ARRAY | SCALAR] [WITH UNIQUE KEYS]`. `IS JSON VALUE` parses to
/// the same node as `IS JSON`, and `WITHOUT UNIQUE KEYS` to the same as no clause. The grammar has
/// no FORMAT clause here, so the node's format is always the default.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L15394
pub(super) fn emit_json_is_predicate(e: &mut EventEmitter, n: &JsonIsPredicate) {
    e.group_start();

    if let Some(ref expr) = n.expr {
        emit_node(e, expr);
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();
    e.token(TokenKind::JSON_KW);

    match n.item_type() {
        JsonValueType::Undefined | JsonValueType::JsTypeAny => {}
        JsonValueType::JsTypeObject => {
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        JsonValueType::JsTypeArray => {
            e.space();
            e.token(TokenKind::ARRAY_KW);
        }
        JsonValueType::JsTypeScalar => {
            e.space();
            e.token(TokenKind::SCALAR_KW);
        }
    }

    if n.unique_keys {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
    }

    e.group_end();
}
