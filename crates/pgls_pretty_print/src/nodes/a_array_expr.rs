use pgls_query::protobuf::AArrayExpr;

use super::prelude::*;

pub(super) fn emit_a_array_expr(e: &mut EventEmitter, n: &AArrayExpr) {
    e.group_start(GroupKind::AArrayExpr);

    e.token(TokenKind::ARRAY_KW);
    e.token(TokenKind::L_BRACK);

    // Laid out like an `IN (...)` list: on one line when it fits, otherwise the elements fill
    // indented lines and `]` goes back to the line the array started on.
    if !n.elements.is_empty() {
        e.indent_start();
        e.line(LineType::Soft);
        e.group_start(GroupKind::List);
        emit_fill_comma_separated_list(e, &n.elements, emit_node);
        e.group_end();
        e.indent_end();
        e.line(LineType::Soft);
    }

    e.token(TokenKind::R_BRACK);

    e.group_end();
}
