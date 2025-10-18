use pgt_query::protobuf::FieldSelect;

use crate::emitter::EventEmitter;

pub(super) fn emit_field_select(e: &mut EventEmitter, n: &FieldSelect) {
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }
}
