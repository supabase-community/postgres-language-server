use pgls_query::protobuf::TableFunc;

use super::prelude::*;

pub(super) fn emit_table_func(e: &mut EventEmitter, n: &TableFunc) {
    e.group_start();

    if let Some(ref expr) = n.docexpr {
        emit_node(e, expr);
    } else if let Some(ref row) = n.rowexpr {
        emit_node(e, row);
    } else {
        emit_identifier(e, &format!("tablefunc#{}", n.ordinalitycol));
    }

    e.group_end();
}
