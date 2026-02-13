use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::JsonAggConstructor;

use super::json_value_expr::emit_json_output;

pub(super) fn emit_json_agg_constructor(e: &mut EventEmitter, n: &JsonAggConstructor) {
    e.group_start(GroupKind::JsonAggConstructor);

    let mut has_content = false;

    if !n.agg_order.is_empty() {
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &n.agg_order, super::emit_node);
        has_content = true;
    }

    emit_json_agg_tail(e, n, has_content);

    e.group_end();
}

pub(super) fn emit_json_agg_tail(
    e: &mut EventEmitter,
    constructor: &JsonAggConstructor,
    mut has_content: bool,
) {
    if let Some(ref output) = constructor.output {
        emit_json_output(e, output, &mut has_content);
    }

    if let Some(ref filter) = constructor.agg_filter {
        if has_content {
            e.line(LineType::SoftOrSpace);
        }
        e.token(TokenKind::FILTER_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, filter);
        e.token(TokenKind::R_PAREN);
        has_content = true;
    }

    if let Some(ref over) = constructor.over {
        if has_content {
            e.line(LineType::SoftOrSpace);
        }
        e.token(TokenKind::OVER_KW);
        e.space();
        super::emit_window_def(e, over);
    }
}
