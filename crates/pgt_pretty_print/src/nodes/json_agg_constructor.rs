use crate::{TokenKind, emitter::EventEmitter};
use pgt_query::protobuf::JsonAggConstructor;

use super::json_value_expr::emit_json_output;

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
            e.space();
        }
        e.token(TokenKind::FILTER_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::WHERE_KW);
        e.space();
        super::emit_node(filter, e);
        e.token(TokenKind::R_PAREN);
        has_content = true;
    }

    if let Some(ref over) = constructor.over {
        if has_content {
            e.space();
        }
        e.token(TokenKind::OVER_KW);
        e.space();
        super::emit_window_def(e, over);
    }
}
