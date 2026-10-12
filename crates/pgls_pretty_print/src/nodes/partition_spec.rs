use pgls_query::protobuf::PartitionSpec;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_parenthesized_list;

pub(super) fn emit_partition_spec(e: &mut EventEmitter, n: &PartitionSpec) {
    e.group_start(GroupKind::PartitionSpec);

    e.token(TokenKind::PARTITION_KW);
    e.space();
    e.token(TokenKind::BY_KW);
    e.space();

    // Emit partition strategy
    // PartitionStrategy: Undefined = 0, List = 1, Range = 2, Hash = 3
    match n.strategy {
        1 => super::emit_word(e, "list"),
        2 => e.token(TokenKind::RANGE_KW),
        3 => super::emit_word(e, "hash"),
        _ => {}
    }

    // Emit partition parameters (columns/expressions)
    if !n.part_params.is_empty() {
        emit_parenthesized_list(e, &n.part_params, super::emit_node);
    }

    e.group_end();
}
