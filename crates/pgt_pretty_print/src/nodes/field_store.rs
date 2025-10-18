use pgt_query::protobuf::FieldStore;

use crate::emitter::EventEmitter;

pub(super) fn emit_field_store(e: &mut EventEmitter, n: &FieldStore) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
