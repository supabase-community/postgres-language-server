use pgls_query::protobuf::{JsonArrayAgg, JsonArrayConstructor, JsonArrayQueryConstructor};

use super::emit_json_value_expr;
use super::json_value_expr::emit_json_output;
use super::prelude::*;

pub(super) fn emit_json_array_constructor(e: &mut EventEmitter, n: &JsonArrayConstructor) {
    e.group_start();

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    if !n.exprs.is_empty() {
        emit_comma_separated_list(e, &n.exprs, |emitter, node| {
            if let Some(pgls_query::NodeEnum::JsonValueExpr(value)) = node.node.as_ref() {
                emit_json_value_expr(emitter, value);
            } else {
                emit_node(emitter, node);
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
    e.group_start();

    e.token(TokenKind::JSON_ARRAY_KW);
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if let Some(ref query) = n.query {
        emit_node(e, query);
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
    e.group_start();

    // Laid out like a function call: the arguments and clauses on one line when they fit,
    // otherwise on their own lines one level in, with `)` back on the call's line. The call has
    // its own group so that FILTER and OVER break before it does.
    let has_suffix = n
        .constructor
        .as_ref()
        .is_some_and(|constructor| constructor.agg_filter.is_some() || constructor.over.is_some());
    if has_suffix {
        e.group_start();
    }
    e.token(TokenKind::JSON_ARRAYAGG_KW);
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(crate::emitter::LineType::Soft);

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
        emit_comma_separated_list(e, &constructor.agg_order, emit_node);
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
        if n.arg.is_some() {
            e.line(crate::emitter::LineType::SoftOrSpace);
        }
        // The line above already separates RETURNING from the arguments.
        emit_json_output(e, output, &mut false);
    }

    e.indent_end();
    e.line(crate::emitter::LineType::Soft);
    e.token(TokenKind::R_PAREN);
    if has_suffix {
        e.group_end();
    }

    // FILTER and OVER clauses go outside the parentheses
    if let Some(ref constructor) = n.constructor {
        if let Some(ref filter) = constructor.agg_filter {
            // A clause that breaks onto its own line continues the call, one level in.
            e.indent_start();
            e.line(crate::emitter::LineType::SoftOrSpace);
            emit_filter_clause(e, filter);
            e.indent_end();
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
