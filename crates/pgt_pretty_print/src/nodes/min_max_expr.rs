use pgt_query::protobuf::{MinMaxExpr, MinMaxOp};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_min_max_expr(e: &mut EventEmitter, n: &MinMaxExpr) {
    e.group_start(GroupKind::MinMaxExpr);

    // MinMaxOp: 0 = GREATEST, 1 = LEAST
    match n.op() {
        MinMaxOp::IsGreatest => e.token(TokenKind::GREATEST_KW),
        MinMaxOp::IsLeast => e.token(TokenKind::LEAST_KW),
        MinMaxOp::Undefined => e.token(TokenKind::IDENT("UNDEFINED_MINMAX".to_string())),
    }

    e.token(TokenKind::L_PAREN);

    if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, super::emit_node);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
