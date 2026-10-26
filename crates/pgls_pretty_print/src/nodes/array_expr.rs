use pgls_query::protobuf::ArrayExpr;

use super::prelude::*;

pub(super) fn emit_array_expr(e: &mut EventEmitter, n: &ArrayExpr) {
    e.group_start();

    e.token(TokenKind::ARRAY_KW);
    e.token(TokenKind::L_BRACK);

    if !n.elements.is_empty() {
        emit_comma_separated_list(e, &n.elements, emit_node);
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
