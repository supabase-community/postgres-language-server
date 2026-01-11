use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{JsonObjectAgg, JsonObjectConstructor};

pub(super) fn emit_json_object_constructor(e: &mut EventEmitter, n: &JsonObjectConstructor) {
    e.group_start(GroupKind::JsonObjectConstructor);

    e.token(TokenKind::JSON_OBJECT_KW);
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

    e.token(TokenKind::JSON_OBJECTAGG_KW);
    e.token(TokenKind::L_PAREN);

    let has_arg = n.arg.is_some();

    if let Some(ref arg) = n.arg {
        super::json_key_value::emit_json_key_value(e, arg);
    }

    if let Some(ref constructor) = n.constructor {
        if !constructor.agg_order.is_empty() {
            if has_arg {
                e.line(crate::emitter::LineType::SoftOrSpace);
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

    // NULL ON NULL / ABSENT ON NULL goes inside the parentheses
    if has_arg {
        e.line(crate::emitter::LineType::SoftOrSpace);
        if n.absent_on_null {
            e.token(TokenKind::ABSENT_KW);
        } else {
            e.token(TokenKind::NULL_KW);
        }
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::NULL_KW);
    }

    if n.unique {
        e.line(crate::emitter::LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::UNIQUE_KW);
        e.space();
        e.token(TokenKind::KEYS_KW);
    }

    // RETURNING clause goes inside the parentheses
    if let Some(ref constructor) = n.constructor {
        if let Some(ref output) = constructor.output {
            let mut has_content = has_arg;
            if has_content {
                e.line(crate::emitter::LineType::SoftOrSpace);
            }
            super::json_value_expr::emit_json_output(e, output, &mut has_content);
        }
    }

    e.token(TokenKind::R_PAREN);

    // FILTER and OVER clauses go outside the parentheses
    if let Some(ref constructor) = n.constructor {
        if let Some(ref filter) = constructor.agg_filter {
            e.line(crate::emitter::LineType::SoftOrSpace);
            e.token(TokenKind::FILTER_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::WHERE_KW);
            super::emit_clause_condition(e, filter);
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref over) = constructor.over {
            e.line(crate::emitter::LineType::SoftOrSpace);
            e.token(TokenKind::OVER_KW);
            e.space();
            super::emit_window_def(e, over);
        }
    }

    e.group_end();
}
