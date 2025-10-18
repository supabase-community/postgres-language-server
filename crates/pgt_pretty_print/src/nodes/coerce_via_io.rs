use pgt_query::protobuf::CoerceViaIo;

use crate::emitter::EventEmitter;

pub(super) fn emit_coerce_via_io(e: &mut EventEmitter, n: &CoerceViaIo) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
