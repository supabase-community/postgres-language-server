use pgls_query::protobuf::RangeTableSample;

use super::prelude::*;

pub(super) fn emit_range_table_sample(e: &mut EventEmitter, n: &RangeTableSample) {
    e.group_start(GroupKind::RangeTableSample);

    // Relation (table)
    if let Some(ref relation) = n.relation {
        emit_node(relation, e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::TABLESAMPLE_KW);
    e.space();

    // Sampling method
    emit_dot_separated_list(e, &n.method);

    // Arguments for the sampling method
    if !n.args.is_empty() {
        e.space();
        emit_parenthesized_list(e, &n.args, emit_node);
    }

    // REPEATABLE clause
    if let Some(ref repeatable) = n.repeatable {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::REPEATABLE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_node(repeatable, e);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
