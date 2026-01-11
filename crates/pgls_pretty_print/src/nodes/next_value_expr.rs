use pgls_query::protobuf::NextValueExpr;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_next_value_expr(e: &mut EventEmitter, n: &NextValueExpr) {
    e.group_start(GroupKind::NextValueExpr);

    let placeholder = format!("nextval#{}", n.seqid);
    super::emit_identifier(e, &placeholder);

    e.group_end();
}
