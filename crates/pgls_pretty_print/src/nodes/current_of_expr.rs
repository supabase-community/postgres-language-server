use pgls_query::protobuf::CurrentOfExpr;

use super::prelude::*;

pub(super) fn emit_current_of_expr(e: &mut EventEmitter, n: &CurrentOfExpr) {
    e.group_start();

    e.token(TokenKind::CURRENT_KW);
    e.space();
    e.token(TokenKind::OF_KW);

    if !n.cursor_name.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.cursor_name);
    }

    e.group_end();
}
