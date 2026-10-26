use pgls_query::protobuf::NextValueExpr;

use super::prelude::*;

pub(super) fn emit_next_value_expr(e: &mut EventEmitter, n: &NextValueExpr) {
    e.group_start();

    let placeholder = format!("nextval#{}", n.seqid);
    emit_identifier(e, &placeholder);

    e.group_end();
}
