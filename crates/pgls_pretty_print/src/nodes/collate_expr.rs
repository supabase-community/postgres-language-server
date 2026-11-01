use pgls_query::protobuf::CollateExpr;

use super::prelude::*;

pub(super) fn emit_collate_expr(e: &mut EventEmitter, n: &CollateExpr) {
    e.group_start();

    if let Some(ref arg) = n.arg {
        emit_node(e, arg);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::COLLATE_KW);
    e.space();

    if n.coll_oid == 0 {
        e.token(TokenKind::DEFAULT_KW);
    } else {
        let placeholder = format!("coll#{}", n.coll_oid);
        emit_identifier(e, &placeholder);
    }

    e.group_end();
}
