use pgls_query::protobuf::DistinctExpr;

use super::prelude::*;

/// Emit a DistinctExpr (planner IS DISTINCT FROM node)
pub(super) fn emit_distinct_expr(e: &mut EventEmitter, n: &DistinctExpr) {
    e.group_start(GroupKind::DistinctExpr);

    // DistinctExpr is planner form of IS DISTINCT FROM
    if n.args.len() == 2 {
        emit_node(&n.args[0], e);
        e.space();
        e.token(TokenKind::IS_KW);
        e.space();
        e.token(TokenKind::DISTINCT_KW);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        emit_node(&n.args[1], e);
    } else {
        emit_args_as_sequence(e, &n.args, n.opno);
    }

    e.group_end();
}
