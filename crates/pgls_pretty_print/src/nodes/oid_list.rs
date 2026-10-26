use pgls_query::protobuf::OidList;

use super::prelude::*;

pub(super) fn emit_oid_list(e: &mut EventEmitter, n: &OidList) {
    e.group_start();

    emit_parenthesized_list(e, &n.items, emit_node);

    e.group_end();
}
