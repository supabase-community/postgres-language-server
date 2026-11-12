use pgls_query::protobuf::FromExpr;

use super::prelude::*;

pub(super) fn emit_from_expr(e: &mut EventEmitter, n: &FromExpr) {
    e.group_start();

    if !n.fromlist.is_empty() {
        emit_comma_separated_list(e, &n.fromlist, emit_node);
    }

    if let Some(ref quals) = n.quals {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        emit_clause_condition(e, quals);
    }

    e.group_end();
}
