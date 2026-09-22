use pgls_query::protobuf::{BoolTestType, BooleanTest};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_boolean_test(e: &mut EventEmitter, n: &BooleanTest) {
    e.group_start(GroupKind::BooleanTest);

    // Emit the argument
    if let Some(ref arg) = n.arg {
        // AND, OR and NOT bind more loosely than the postfix IS test, so the grouping is lost
        // unless it is spelled out: `(a OR b) IS TRUE` would come back as `a OR b IS TRUE`, which
        // parses as `a OR (b IS TRUE)`. Every other argument kind binds tighter and needs nothing.
        let needs_parens = matches!(arg.node.as_ref(), Some(pgls_query::NodeEnum::BoolExpr(_)));

        if needs_parens {
            e.token(TokenKind::L_PAREN);
        }

        super::emit_node(arg, e);

        if needs_parens {
            e.token(TokenKind::R_PAREN);
        }
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
