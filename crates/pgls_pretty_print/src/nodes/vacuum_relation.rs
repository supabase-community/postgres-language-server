use pgls_query::protobuf::VacuumRelation;

use super::prelude::*;

pub(super) fn emit_vacuum_relation(e: &mut EventEmitter, n: &VacuumRelation) {
    e.group_start();

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // va_cols: specific columns to vacuum/analyze
    if !n.va_cols.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.va_cols, emit_node);
    }

    e.group_end();
}
