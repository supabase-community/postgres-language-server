use pgls_query::protobuf::OidList;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_oid_list(e: &mut EventEmitter, n: &OidList) {
    e.group_start(GroupKind::OidList);

    super::node_list::emit_parenthesized_list(e, &n.items, super::emit_node);

    e.group_end();
}
