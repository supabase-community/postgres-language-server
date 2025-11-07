use pgls_query::protobuf::InferenceElem;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_inference_elem(e: &mut EventEmitter, n: &InferenceElem) {
    e.group_start(GroupKind::InferenceElem);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    } else if let Some(ref xpr) = n.xpr {
        super::emit_node(xpr, e);
    }

    e.group_end();
}
