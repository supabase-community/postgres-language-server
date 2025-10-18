use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::JsonIsPredicate;

pub(super) fn emit_json_is_predicate(e: &mut EventEmitter, n: &JsonIsPredicate) {
    e.group_start(GroupKind::JsonIsPredicate);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();

    // item_type: JsTypeAny = 0, JsTypeObject = 1, JsTypeArray = 2, JsTypeScalar = 3
    match n.item_type {
        0 => e.token(TokenKind::IDENT("JSON".to_string())),
        1 => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("OBJECT".to_string()));
        }
        2 => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ARRAY".to_string()));
        }
        3 => {
            e.token(TokenKind::IDENT("JSON".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SCALAR".to_string()));
        }
        _ => e.token(TokenKind::IDENT("JSON".to_string())),
    }

    e.group_end();
}
