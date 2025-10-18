use pgt_query::protobuf::CurrentOfExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_current_of_expr(e: &mut EventEmitter, n: &CurrentOfExpr) {
    e.group_start(GroupKind::CurrentOfExpr);

    e.token(TokenKind::CURRENT_KW);
    e.space();
    e.token(TokenKind::OF_KW);

    if !n.cursor_name.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.cursor_name.clone()));
    }

    e.group_end();
}
