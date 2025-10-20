use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{JsonObjectAgg, JsonObjectConstructor};

use super::json_agg_constructor::emit_json_agg_tail;

pub(super) fn emit_json_object_constructor(e: &mut EventEmitter, n: &JsonObjectConstructor) {
    e.group_start(GroupKind::JsonObjectConstructor);

    e.token(TokenKind::IDENT("JSON_OBJECT".to_string()));
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.exprs.is_empty() {
        super::node_list::emit_comma_separated_list(e, &n.exprs, super::emit_node);
        has_content = true;
    }

    if n.absent_on_null {
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

    if n.unique {
        if has_content {
            e.space();
        }
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
        has_content = true;
    }

    if let Some(ref output) = n.output {
        super::json_value_expr::emit_json_output(e, output, &mut has_content);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}

pub(super) fn emit_json_object_agg(e: &mut EventEmitter, n: &JsonObjectAgg) {
    e.group_start(GroupKind::JsonObjectAgg);

    e.token(TokenKind::IDENT("JSON_OBJECTAGG".to_string()));
    e.token(TokenKind::L_PAREN);

    if let Some(ref arg) = n.arg {
        super::json_key_value::emit_json_key_value(e, arg);
    }

    if let Some(ref constructor) = n.constructor {
        if !constructor.agg_order.is_empty() {
            if n.arg.is_some() {
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

    if n.unique {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
    }

    if let Some(ref constructor) = n.constructor {
        emit_json_agg_tail(e, constructor, true);
    }

    e.group_end();
}
