use pgls_query::protobuf::FromExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_from_expr(e: &mut EventEmitter, n: &FromExpr) {
    e.group_start(GroupKind::FromExpr);

    if !n.fromlist.is_empty() {
        emit_comma_separated_list(e, &n.fromlist, super::emit_node);
    }

    if let Some(ref quals) = n.quals {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, quals);
    }

    e.group_end();
}
