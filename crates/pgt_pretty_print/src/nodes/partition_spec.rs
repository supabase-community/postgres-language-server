use pgt_query::protobuf::PartitionSpec;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_partition_spec(e: &mut EventEmitter, n: &PartitionSpec) {
    e.group_start(GroupKind::PartitionSpec);

    e.token(TokenKind::PARTITION_KW);
    e.space();
    e.token(TokenKind::BY_KW);
    e.space();

    // Emit partition strategy
    // PartitionStrategy: Undefined = 0, List = 1, Range = 2, Hash = 3
    match n.strategy {
        1 => e.token(TokenKind::IDENT("LIST".to_string())),
        2 => e.token(TokenKind::RANGE_KW),
        3 => e.token(TokenKind::IDENT("HASH".to_string())),
        _ => {}
    }

    // Emit partition parameters (columns/expressions)
    if !n.part_params.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.part_params, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}
