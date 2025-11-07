use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::StatsElem;

pub(super) fn emit_stats_elem(e: &mut EventEmitter, n: &StatsElem) {
    e.group_start(GroupKind::StatsElem);

    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    } else if !n.name.is_empty() {
        super::emit_identifier(e, &n.name);
    }

    e.group_end();
}
