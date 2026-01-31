use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::TableFunc;

pub(super) fn emit_table_func(e: &mut EventEmitter, n: &TableFunc) {
    e.group_start(GroupKind::TableFunc);

    if let Some(ref expr) = n.docexpr {
        super::emit_node(expr, e);
    } else if let Some(ref row) = n.rowexpr {
        super::emit_node(row, e);
    } else {
        super::emit_identifier(e, &format!("tablefunc#{}", n.ordinalitycol));
    }

    e.group_end();
}
