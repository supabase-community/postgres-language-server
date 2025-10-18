use pgt_query::protobuf::VacuumRelation;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::emit_node;

pub(super) fn emit_vacuum_relation(e: &mut EventEmitter, n: &VacuumRelation) {
    e.group_start(GroupKind::VacuumRelation);

    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // va_cols: specific columns to vacuum/analyze
    if !n.va_cols.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.va_cols, emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
