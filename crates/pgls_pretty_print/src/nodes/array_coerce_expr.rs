use pgls_query::protobuf::ArrayCoerceExpr;

use super::prelude::*;

pub(super) fn emit_array_coerce_expr(e: &mut EventEmitter, n: &ArrayCoerceExpr) {
    if let Some(ref arg) = n.arg {
        emit_node(e, arg);
    }
}
