use pgls_query::protobuf::CoerceToDomain;

use super::prelude::*;

pub(super) fn emit_coerce_to_domain(e: &mut EventEmitter, n: &CoerceToDomain) {
    if let Some(ref arg) = n.arg {
        emit_node(arg, e);
    }
}
