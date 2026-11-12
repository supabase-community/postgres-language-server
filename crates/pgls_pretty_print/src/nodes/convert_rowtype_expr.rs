use pgls_query::protobuf::ConvertRowtypeExpr;

use super::prelude::*;

pub(super) fn emit_convert_rowtype_expr(e: &mut EventEmitter, n: &ConvertRowtypeExpr) {
    if let Some(ref arg) = n.arg {
        emit_node(e, arg);
    }
}
