use pgls_query::protobuf::NullIfExpr;

use super::prelude::*;

/// Emit a NullIfExpr (planner NULLIF node)
pub(super) fn emit_null_if_expr(e: &mut EventEmitter, n: &NullIfExpr) {
    e.group_start();

    // NullIfExpr is planner form of NULLIF(a, b)
    e.token(TokenKind::NULLIF_KW);
    e.token(TokenKind::L_PAREN);
    if let [first, second, ..] = n.args.as_slice() {
        emit_node(e, first);
        emit_argument_comma(e, Some(second));
        e.space();
        emit_node(e, second);
    } else {
        emit_args_as_sequence(e, &n.args, n.opno);
    }
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
