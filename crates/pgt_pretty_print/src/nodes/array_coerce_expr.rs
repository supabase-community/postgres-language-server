use pgt_query::protobuf::ArrayCoerceExpr;

use crate::emitter::EventEmitter;

pub(super) fn emit_array_coerce_expr(e: &mut EventEmitter, n: &ArrayCoerceExpr) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
