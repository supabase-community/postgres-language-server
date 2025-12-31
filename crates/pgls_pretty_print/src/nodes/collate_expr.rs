use pgls_query::protobuf::CollateExpr;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_collate_expr(e: &mut EventEmitter, n: &CollateExpr) {
    e.group_start(GroupKind::CollateExpr);

    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::COLLATE_KW);
    e.space();

    if n.coll_oid == 0 {
        e.token(TokenKind::DEFAULT_KW);
    } else {
        let placeholder = format!("coll#{}", n.coll_oid);
        super::emit_identifier(e, &placeholder);
    }

    e.group_end();
}
