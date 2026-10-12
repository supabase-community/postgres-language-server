use pgls_query::protobuf::IntList;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_int_list(e: &mut EventEmitter, n: &IntList) {
    e.group_start(GroupKind::IntList);

    super::node_list::emit_parenthesized_list(e, &n.items, super::emit_node);

    e.group_end();
}
