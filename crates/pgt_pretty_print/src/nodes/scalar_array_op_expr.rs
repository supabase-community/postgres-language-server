use pgt_query::protobuf::ScalarArrayOpExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_scalar_array_op_expr(e: &mut EventEmitter, n: &ScalarArrayOpExpr) {
    e.group_start(GroupKind::ScalarArrayOpExpr);

    debug_assert!(
        n.args.len() == 2,
        "ScalarArrayOpExpr should have exactly two arguments"
    );

    if n.args.len() == 2 {
        let lhs = &n.args[0];
        let rhs = &n.args[1];

        super::emit_node(lhs, e);
        e.space();

        // TODO: derive operator token from opno instead of assuming equality.
        e.token(TokenKind::IDENT("=".to_string()));
        e.space();

        if n.use_or {
            e.token(TokenKind::ANY_KW);
        } else {
            e.token(TokenKind::ALL_KW);
        }
        e.space();

        e.token(TokenKind::L_PAREN);
        super::emit_node(rhs, e);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
