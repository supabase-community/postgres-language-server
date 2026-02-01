use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::{node_list::emit_comma_separated_list, node_list::emit_dot_separated_list},
};
use pgls_query::protobuf::RangeTableSample;

pub(super) fn emit_range_table_sample(e: &mut EventEmitter, n: &RangeTableSample) {
    e.group_start(GroupKind::RangeTableSample);

    // Relation (table)
    if let Some(ref relation) = n.relation {
        super::emit_node(relation, e);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::TABLESAMPLE_KW);
    e.space();

    // Sampling method
    emit_dot_separated_list(e, &n.method);

    // Arguments for the sampling method
    if !n.args.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.args, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // REPEATABLE clause
    if let Some(ref repeatable) = n.repeatable {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::REPEATABLE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        super::emit_node(repeatable, e);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
