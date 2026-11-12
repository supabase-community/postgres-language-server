use pgls_query::protobuf::FieldStore;

use super::prelude::*;

pub(super) fn emit_field_store(e: &mut EventEmitter, n: &FieldStore) {
    if let Some(ref arg) = n.arg {
        emit_node(arg, e);
    }
}
