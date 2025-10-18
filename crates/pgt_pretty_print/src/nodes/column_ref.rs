use pgt_query::protobuf::ColumnRef;

use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_column_ref(e: &mut EventEmitter, n: &ColumnRef) {
    e.group_start(GroupKind::ColumnRef);
    emit_dot_separated_list(e, &n.fields);
    e.group_end();
}
