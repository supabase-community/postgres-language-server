use pgls_query::protobuf::JsonObjectAgg;

use super::prelude::*;

pub(super) fn emit_json_object_agg(e: &mut EventEmitter, n: &JsonObjectAgg) {
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
    e.token(TokenKind::JSON_OBJECTAGG_KW);
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(crate::emitter::LineType::Soft);

    let has_arg = n.arg.is_some();

    if let Some(ref arg) = n.arg {
        super::emit_json_key_value(e, arg);
    }

    if let Some(ref constructor) = n.constructor
        && !constructor.agg_order.is_empty()
    {
        if has_arg {
            e.line(crate::emitter::LineType::SoftOrSpace);
        }
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &constructor.agg_order, emit_node);
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
    if let Some(ref constructor) = n.constructor
        && let Some(ref output) = constructor.output
    {
        if has_arg {
            e.line(crate::emitter::LineType::SoftOrSpace);
        }
        // The line above already separates RETURNING from the arguments.
        emit_json_output_clause(e, output, &mut false);
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
