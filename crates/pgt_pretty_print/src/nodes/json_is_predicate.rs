use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{JsonIsPredicate, JsonValueType};

pub(super) fn emit_json_is_predicate(e: &mut EventEmitter, n: &JsonIsPredicate) {
    e.group_start(GroupKind::JsonIsPredicate);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();

    match n.item_type() {
        JsonValueType::Undefined | JsonValueType::JsTypeAny => {
            e.token(TokenKind::IDENT("JSON".to_string()))
        }
        JsonValueType::JsTypeObject => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("OBJECT".to_string()));
        }
        JsonValueType::JsTypeArray => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ARRAY".to_string()));
        }
        JsonValueType::JsTypeScalar => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SCALAR".to_string()));
        }
    }

    e.group_end();
}
