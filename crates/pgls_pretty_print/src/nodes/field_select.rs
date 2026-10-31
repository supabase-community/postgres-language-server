use pgls_query::protobuf::FieldSelect;

use super::prelude::*;

pub(super) fn emit_field_select(e: &mut EventEmitter, n: &FieldSelect) {
    if let Some(ref arg) = n.arg {
        emit_node(arg, e);
    }
}
