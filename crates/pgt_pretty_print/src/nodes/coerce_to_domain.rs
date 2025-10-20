use pgt_query::protobuf::CoerceToDomain;

use crate::emitter::EventEmitter;

pub(super) fn emit_coerce_to_domain(e: &mut EventEmitter, n: &CoerceToDomain) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
