use pgls_query::protobuf::RangeTblRef;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_range_tbl_ref(e: &mut EventEmitter, n: &RangeTblRef) {
    e.group_start(GroupKind::RangeTblRef);

    super::emit_identifier(e, &format!("rte#{}", n.rtindex));

    e.group_end();
}
