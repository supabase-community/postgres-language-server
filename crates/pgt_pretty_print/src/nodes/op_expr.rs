use pgt_query::protobuf::Node;
use pgt_query::protobuf::{DistinctExpr, NullIfExpr, OpExpr};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

/// Emit an OpExpr (planner operator node)
/// These are internal planner representations with operator OIDs
/// We emit a simple fallback since we don't have access to operator names
pub(super) fn emit_op_expr(e: &mut EventEmitter, n: &OpExpr) {
    e.group_start(GroupKind::OpExpr);

    // OpExpr represents binary operators in the planner
    // Without a pg_operator lookup, we use a generic fallback
    if n.args.len() == 2 {
        super::emit_node(&n.args[0], e);
        e.space();
        // opno is the operator OID - we don't have the symbol
        e.token(TokenKind::IDENT(format!("op#{}", n.opno)));
        e.space();
        super::emit_node(&n.args[1], e);
    } else {
        // Fallback for unexpected arg counts
        emit_args_as_sequence(e, &n.args, n.opno);
    }

    e.group_end();
}

/// Emit a DistinctExpr (planner IS DISTINCT FROM node)
pub(super) fn emit_distinct_expr(e: &mut EventEmitter, n: &DistinctExpr) {
    e.group_start(GroupKind::DistinctExpr);

    // DistinctExpr is planner form of IS DISTINCT FROM
    if n.args.len() == 2 {
        super::emit_node(&n.args[0], e);
        e.space();
        e.token(TokenKind::IS_KW);
        e.space();
        e.token(TokenKind::DISTINCT_KW);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        super::emit_node(&n.args[1], e);
    } else {
        emit_args_as_sequence(e, &n.args, n.opno);
    }

    e.group_end();
}

/// Emit a NullIfExpr (planner NULLIF node)
pub(super) fn emit_null_if_expr(e: &mut EventEmitter, n: &NullIfExpr) {
    e.group_start(GroupKind::NullIfExpr);

    // NullIfExpr is planner form of NULLIF(a, b)
    e.token(TokenKind::IDENT("NULLIF".to_string()));
    e.token(TokenKind::L_PAREN);
    if n.args.len() >= 2 {
        super::emit_node(&n.args[0], e);
        e.token(TokenKind::COMMA);
        e.space();
        super::emit_node(&n.args[1], e);
    } else {
        emit_args_as_sequence(e, &n.args, n.opno);
    }
    e.token(TokenKind::R_PAREN);

    e.group_end();
}

fn emit_args_as_sequence(e: &mut EventEmitter, args: &[Node], opno: u32) {
    if args.is_empty() {
        e.token(TokenKind::IDENT(format!("op#{}", opno)));
        return;
    }

    if args.len() == 1 {
        e.token(TokenKind::IDENT(format!("op#{}", opno)));
        e.space();
        super::emit_node(&args[0], e);
        return;
    }

    super::emit_node(&args[0], e);
    for arg in &args[1..] {
        e.space();
        e.token(TokenKind::IDENT(format!("op#{}", opno)));
        e.space();
        super::emit_node(arg, e);
    }
}
