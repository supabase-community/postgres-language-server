use pgls_query::protobuf::OpExpr;

use super::prelude::*;

/// Emit an OpExpr (planner operator node)
/// These are internal planner representations with operator OIDs
/// We emit a simple fallback since we don't have access to operator names
pub(super) fn emit_op_expr(e: &mut EventEmitter, n: &OpExpr) {
    e.group_start();

    // OpExpr represents binary operators in the planner
    // Without a pg_operator lookup, we use a generic fallback
    if let [first, second] = n.args.as_slice() {
        emit_node(e, first);
        e.space();
        // opno is the operator OID - we don't have the symbol
        e.token(TokenKind::IDENT(format!("op#{}", n.opno)));
        e.space();
        emit_node(e, second);
    } else {
        // Fallback for unexpected arg counts
        emit_args_as_sequence(e, &n.args, n.opno);
    }

    e.group_end();
}
