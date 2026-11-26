use pgls_query::protobuf::{BoolTestType, BooleanTest};

use super::prelude::*;

pub(super) fn emit_boolean_test(e: &mut EventEmitter, n: &BooleanTest) {
    e.group_start();

    if let Some(ref arg) = n.arg {
        emit_operand_with_parens(e, arg, Some(OperatorInfo::IS), OperandSide::Left);
    }

    e.line(LineType::SoftOrSpace);
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
