use pgls_query::protobuf::InferenceElem;

use super::prelude::*;

pub(super) fn emit_inference_elem(e: &mut EventEmitter, n: &InferenceElem) {
    e.group_start();

    if let Some(ref expr) = n.expr {
        emit_node(e, expr);
    } else if let Some(ref xpr) = n.xpr {
        emit_node(e, xpr);
    }

    e.group_end();
}
