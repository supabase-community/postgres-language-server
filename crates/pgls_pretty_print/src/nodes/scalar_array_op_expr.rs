use pgls_query::protobuf::ScalarArrayOpExpr;

use super::prelude::*;

pub(super) fn emit_scalar_array_op_expr(e: &mut EventEmitter, n: &ScalarArrayOpExpr) {
    e.group_start();

    debug_assert!(
        n.args.len() == 2,
        "ScalarArrayOpExpr should have exactly two arguments"
    );

    if n.args.len() == 2 {
        let lhs = &n.args[0];
        let rhs = &n.args[1];

        emit_node(e, lhs);
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
        emit_node(e, rhs);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
