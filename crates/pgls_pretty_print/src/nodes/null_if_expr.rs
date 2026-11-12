use pgls_query::protobuf::NullIfExpr;

use super::prelude::*;

/// Emit a NullIfExpr (planner NULLIF node)
pub(super) fn emit_null_if_expr(e: &mut EventEmitter, n: &NullIfExpr) {
    e.group_start();

    // NullIfExpr is planner form of NULLIF(a, b)
    e.token(TokenKind::NULLIF_KW);
    e.token(TokenKind::L_PAREN);
    if n.args.len() >= 2 {
        emit_node(e, &n.args[0]);
        e.token(TokenKind::COMMA);
        e.space();
        emit_node(e, &n.args[1]);
    } else {
        emit_args_as_sequence(e, &n.args, n.opno);
    }
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
