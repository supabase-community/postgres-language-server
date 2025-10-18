use pgt_query::protobuf::{BoolTestType, BooleanTest};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_boolean_test(e: &mut EventEmitter, n: &BooleanTest) {
    e.group_start(GroupKind::BooleanTest);

    // Emit the argument
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();

    // Map test type to keywords
    match n.booltesttype() {
        BoolTestType::IsTrue => {
            e.token(TokenKind::TRUE_KW);
        }
        BoolTestType::IsNotTrue => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::TRUE_KW);
        }
        BoolTestType::IsFalse => {
            e.token(TokenKind::FALSE_KW);
        }
        BoolTestType::IsNotFalse => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::FALSE_KW);
        }
        BoolTestType::IsUnknown => {
            e.token(TokenKind::UNKNOWN_KW);
        }
        BoolTestType::IsNotUnknown => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::UNKNOWN_KW);
        }
        BoolTestType::Undefined => {
            // Shouldn't happen, but handle gracefully
            e.token(TokenKind::TRUE_KW);
        }
    }

    e.group_end();
}
