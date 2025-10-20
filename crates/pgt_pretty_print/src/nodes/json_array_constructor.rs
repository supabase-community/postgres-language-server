use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{JsonArrayAgg, JsonArrayConstructor, JsonArrayQueryConstructor};

use super::{
    json_agg_constructor::emit_json_agg_tail,
    json_value_expr::{emit_json_output, emit_json_value_expr},
};

pub(super) fn emit_json_array_constructor(e: &mut EventEmitter, n: &JsonArrayConstructor) {
    e.group_start(GroupKind::JsonArrayConstructor);

    e.token(TokenKind::IDENT("JSON_ARRAY".to_string()));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.exprs.is_empty() {
        super::node_list::emit_comma_separated_list(e, &n.exprs, |node, emitter| {
            if let Some(pgt_query::NodeEnum::JsonValueExpr(value)) = node.node.as_ref() {
                emit_json_value_expr(emitter, value);
            } else {
                super::emit_node(node, emitter);
            }
        });
        has_content = true;
    }

    if n.absent_on_null && !n.exprs.is_empty() {
        if has_content {
            e.space();
        }
        e.token(TokenKind::ABSENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}

pub(super) fn emit_json_array_query_constructor(
    e: &mut EventEmitter,
    n: &JsonArrayQueryConstructor,
) {
    e.group_start(GroupKind::JsonArrayQueryConstructor);

    e.token(TokenKind::IDENT("JSON_ARRAY".to_string()));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref query) = n.query {
        super::emit_node(query, e);
        has_content = true;
    }

    if n.absent_on_null && has_content {
        e.space();
        e.token(TokenKind::ABSENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
        has_content = true;
    }

    if let Some(ref format) = n.format {
        super::json_value_expr::emit_json_format(e, format);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}

pub(super) fn emit_json_array_agg(e: &mut EventEmitter, n: &JsonArrayAgg) {
    e.group_start(GroupKind::JsonArrayAgg);

    e.token(TokenKind::IDENT("JSON_ARRAYAGG".to_string()));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref arg) = n.arg {
        emit_json_value_expr(e, arg);
        has_content = true;
    }

    if let Some(ref constructor) = n.constructor {
        if !constructor.agg_order.is_empty() {
            if has_content {
                e.space();
            }
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            super::node_list::emit_comma_separated_list(
                e,
                &constructor.agg_order,
                super::emit_node,
            );
            has_content = true;
        }
    }

    e.token(TokenKind::R_PAREN);

    if n.absent_on_null {
        e.space();
        e.token(TokenKind::ABSENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
    }

    if let Some(ref constructor) = n.constructor {
        emit_json_agg_tail(e, constructor, true);
    }

    e.group_end();
}
