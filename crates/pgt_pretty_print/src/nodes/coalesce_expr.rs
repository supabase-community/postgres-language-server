use pgt_query::protobuf::CoalesceExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_coalesce_expr(e: &mut EventEmitter, n: &CoalesceExpr) {
    e.group_start(GroupKind::CoalesceExpr);

    e.token(TokenKind::COALESCE_KW);
    e.token(TokenKind::L_PAREN);

    if !n.args.is_empty() {
        emit_comma_separated_list(e, &n.args, super::emit_node);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
