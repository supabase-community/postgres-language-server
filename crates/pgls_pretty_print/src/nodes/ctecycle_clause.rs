use pgls_query::protobuf::CteCycleClause;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_ctecycle_clause(e: &mut EventEmitter, n: &CteCycleClause) {
    e.group_start(GroupKind::CtecycleClause);

    e.token(TokenKind::CYCLE_KW);

    if !n.cycle_col_list.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.cycle_col_list, super::emit_node);
    }

    if !n.cycle_mark_column.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.cycle_mark_column);
    }

    if let Some(ref value) = n.cycle_mark_value {
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        super::emit_node(value, e);
    }

    if let Some(ref default_value) = n.cycle_mark_default {
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        super::emit_node(default_value, e);
    }

    if !n.cycle_path_column.is_empty() {
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        super::emit_identifier_maybe_quoted(e, &n.cycle_path_column);
    }

    e.group_end();
}
