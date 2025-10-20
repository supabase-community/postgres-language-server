use pgt_query::protobuf::RelabelType;

use crate::emitter::EventEmitter;

pub(super) fn emit_relabel_type(e: &mut EventEmitter, n: &RelabelType) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
