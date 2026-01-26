use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{JsonArrayAgg, JsonArrayConstructor, JsonArrayQueryConstructor};

use super::json_value_expr::{emit_json_output, emit_json_value_expr};

pub(super) fn emit_json_array_constructor(e: &mut EventEmitter, n: &JsonArrayConstructor) {
    e.group_start(GroupKind::JsonArrayConstructor);

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    if !n.exprs.is_empty() {
        super::node_list::emit_comma_separated_list(e, &n.exprs, |node, emitter| {
            if let Some(pgls_query::NodeEnum::JsonValueExpr(value)) = node.node.as_ref() {
                emit_json_value_expr(emitter, value);
            } else {
                super::emit_node(node, emitter);
            }
        });
    }

    // The default for JSON_ARRAY without a null clause is ABSENT ON NULL
    // So we only need to emit NULL ON NULL when absent_on_null is false
    if !n.exprs.is_empty() {
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

    if let Some(ref output) = n.output {
        let mut guard = !n.exprs.is_empty();
        emit_json_output(e, output, &mut guard);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}

pub(super) fn emit_json_array_query_constructor(
    e: &mut EventEmitter,
    n: &JsonArrayQueryConstructor,
) {
    e.group_start(GroupKind::JsonArrayQueryConstructor);

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref query) = n.query {
        // Emit select without semicolon since it's inside JSON_ARRAY()
        if let Some(pgls_query::NodeEnum::SelectStmt(select)) = query.node.as_ref() {
            super::select_stmt::emit_select_stmt_no_semicolon(e, select);
        } else {
            super::emit_node(query, e);
        }
        has_content = true;
    }

    // Note: For JSON_ARRAY with subquery, ABSENT ON NULL is always the default
    // and cannot be overridden, so we don't emit it

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

    e.token(TokenKind::JSON_ARRAYAGG_KW);
    e.token(TokenKind::L_PAREN);

    if let Some(ref arg) = n.arg {
        emit_json_value_expr(e, arg);
    }

    if let Some(ref constructor) = n.constructor
        && !constructor.agg_order.is_empty()
    {
        if n.arg.is_some() {
            e.line(crate::emitter::LineType::SoftOrSpace);
        }
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::node_list::emit_comma_separated_list(e, &constructor.agg_order, super::emit_node);
    }

    // NULL ON NULL / ABSENT ON NULL goes inside the parentheses
    if n.arg.is_some() {
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

    // RETURNING clause goes inside the parentheses
    if let Some(ref constructor) = n.constructor
        && let Some(ref output) = constructor.output
    {
        let mut has_content = n.arg.is_some();
        e.line(crate::emitter::LineType::SoftOrSpace);
        emit_json_output(e, output, &mut has_content);
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
