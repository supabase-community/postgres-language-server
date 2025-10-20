use pgt_query::protobuf::ConvertRowtypeExpr;

use crate::emitter::EventEmitter;

pub(super) fn emit_convert_rowtype_expr(e: &mut EventEmitter, n: &ConvertRowtypeExpr) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
