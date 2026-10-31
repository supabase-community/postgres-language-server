use pgls_query::protobuf::RelabelType;

use super::prelude::*;

pub(super) fn emit_relabel_type(e: &mut EventEmitter, n: &RelabelType) {
    if let Some(ref arg) = n.arg {
        emit_node(e, arg);
    }
}
