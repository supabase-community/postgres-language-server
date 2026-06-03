use pgls_query::protobuf::CurrentOfExpr;

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
        super::emit_identifier_maybe_quoted(e, &n.cursor_name);
    }

    e.group_end();
}
