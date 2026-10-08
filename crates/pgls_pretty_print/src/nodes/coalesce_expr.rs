use pgls_query::protobuf::CoalesceExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_coalesce_expr(e: &mut EventEmitter, n: &CoalesceExpr) {
    e.group_start(GroupKind::CoalesceExpr);

    e.token(TokenKind::COALESCE_KW);
    e.token(TokenKind::L_PAREN);

    if !n.args.is_empty() {
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.args, super::emit_node);
        e.indent_end();
        e.line(LineType::Soft);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
