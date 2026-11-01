use pgls_query::protobuf::PartitionBoundSpec;

use super::prelude::*;

pub(super) fn emit_partition_bound_spec(e: &mut EventEmitter, n: &PartitionBoundSpec) {
    e.group_start();

    // Check for DEFAULT partition
    if n.is_default {
        e.token(TokenKind::DEFAULT_KW);
        e.group_end();
        return;
    }

    // FOR VALUES clause
    e.token(TokenKind::FOR_KW);
    e.space();
    e.token(TokenKind::VALUES_KW);

    // Different strategies:
    // 'r' = RANGE: FOR VALUES FROM (...) TO (...)
    // 'l' = LIST: FOR VALUES IN (...)
    // 'h' = HASH: FOR VALUES WITH (MODULUS x, REMAINDER y)
    match n.strategy.as_str() {
        "r" => {
            // RANGE partition
            if !n.lowerdatums.is_empty() {
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
                emit_parenthesized_list(e, &n.lowerdatums, emit_node);
            }
            if !n.upperdatums.is_empty() {
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                emit_parenthesized_list(e, &n.upperdatums, emit_node);
            }
        }
        "l" => {
            // LIST partition
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            emit_parenthesized_list(e, &n.listdatums, emit_node);
        }
        "h" => {
            // HASH partition
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_keyword(e, "modulus");
            e.space();
            e.token(TokenKind::IDENT(n.modulus.to_string()));
            e.token(TokenKind::COMMA);
            e.space();
            emit_keyword(e, "remainder");
            e.space();
            e.token(TokenKind::IDENT(n.remainder.to_string()));
            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Unknown strategy, just emit FOR VALUES
        }
    }

    e.group_end();
}
