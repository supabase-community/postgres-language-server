use pgt_query::protobuf::AArrayExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_a_array_expr(e: &mut EventEmitter, n: &AArrayExpr) {
    e.group_start(GroupKind::AArrayExpr);

    e.token(TokenKind::ARRAY_KW);
    e.token(TokenKind::L_BRACK);

    if !n.elements.is_empty() {
        emit_comma_separated_list(e, &n.elements, super::emit_node);
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
