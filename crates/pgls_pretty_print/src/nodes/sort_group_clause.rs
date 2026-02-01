use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::SortGroupClause;

pub(super) fn emit_sort_group_clause(e: &mut EventEmitter, n: &SortGroupClause) {
    e.group_start(GroupKind::SortGroupClause);

    super::emit_identifier(e, &format!("sortgroup#{}", n.tle_sort_group_ref));

    if n.sortop != 0 {
        e.space();
        super::emit_identifier(e, &format!("sortop#{}", n.sortop));
    }

    if n.eqop != 0 {
        e.space();
        super::emit_identifier(e, &format!("eqop#{}", n.eqop));
    }

    if n.nulls_first {
        e.space();
        super::emit_identifier(e, "nulls_first");
    }

    if n.hashable {
        e.space();
        super::emit_identifier(e, "hashable");
    }

    e.group_end();
}
