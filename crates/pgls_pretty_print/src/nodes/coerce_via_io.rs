use pgls_query::protobuf::CoerceViaIo;

use super::prelude::*;

pub(super) fn emit_coerce_via_io(e: &mut EventEmitter, n: &CoerceViaIo) {
    if let Some(ref arg) = n.arg {
        emit_node(e, arg);
    }
}
