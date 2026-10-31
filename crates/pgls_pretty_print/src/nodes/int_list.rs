use pgls_query::protobuf::IntList;

use super::prelude::*;

pub(super) fn emit_int_list(e: &mut EventEmitter, n: &IntList) {
    e.group_start();

    emit_parenthesized_list(e, &n.items, emit_node);

    e.group_end();
}
