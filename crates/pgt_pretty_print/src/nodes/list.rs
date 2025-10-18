use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::List;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_list(e: &mut EventEmitter, n: &List) {
    e.group_start(GroupKind::List);

    emit_comma_separated_list(e, &n.items, super::emit_node);

    e.group_end();
}
