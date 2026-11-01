use pgls_query::protobuf::{JsonBehavior, JsonBehaviorType, JsonFuncExpr};

use super::prelude::*;

fn emit_json_behavior(e: &mut EventEmitter, behavior: &JsonBehavior) {
    match behavior.btype() {
        JsonBehaviorType::JsonBehaviorError => {
            e.token(TokenKind::ERROR_KW);
        }
        JsonBehaviorType::JsonBehaviorNull => {
            e.token(TokenKind::NULL_KW);
        }
        JsonBehaviorType::JsonBehaviorTrue => {
            e.token(TokenKind::TRUE_KW);
        }
        JsonBehaviorType::JsonBehaviorFalse => {
            e.token(TokenKind::FALSE_KW);
        }
        JsonBehaviorType::JsonBehaviorEmpty => {
            e.token(TokenKind::EMPTY_KW);
        }
        JsonBehaviorType::JsonBehaviorEmptyArray => {
            e.token(TokenKind::EMPTY_KW);
            e.space();
            e.token(TokenKind::ARRAY_KW);
        }
        JsonBehaviorType::JsonBehaviorEmptyObject => {
            e.token(TokenKind::EMPTY_KW);
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        JsonBehaviorType::JsonBehaviorUnknown => {
            e.token(TokenKind::UNKNOWN_KW);
        }
        JsonBehaviorType::JsonBehaviorDefault => {
            e.token(TokenKind::DEFAULT_KW);
            if let Some(ref expr) = behavior.expr {
                e.space();
                emit_node(e, expr);
            }
        }
        _ => {}
    }
}

pub(super) fn emit_json_func_expr(e: &mut EventEmitter, n: &JsonFuncExpr) {
    e.group_start();

    // Map JSON function operation types
    // 0=Undefined, 1=JsonExistsOp, 2=JsonQueryOp, 3=JsonValueOp, 4=JsonTableOp
    let keyword = match n.op {
        1 => TokenKind::JSON_EXISTS_KW,
        2 => TokenKind::JSON_QUERY_KW,
        3 => TokenKind::JSON_VALUE_KW,
        _ => {
            // Unknown JSON function - emit placeholder
            e.token(TokenKind::IDENT("json_func".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
            e.group_end();
            return;
        }
    };
    let is_exists = n.op == 1;
    let is_query = n.op == 2;

    // Laid out like a function call: on one line when it fits, otherwise the context item, the
    // path and every clause on their own line, one level in, and `)` back on the call's line.
    e.token(keyword);
    e.token(TokenKind::L_PAREN);
    e.indent_start();
    e.line(LineType::Soft);

    if let Some(ref context) = n.context_item {
        if let Some(ref raw_expr) = context.raw_expr {
            emit_node(e, raw_expr);
        }
        if let Some(ref format) = context.format {
            super::json_value_expr::emit_json_format(e, format);
        }
    }

    e.token(TokenKind::COMMA);
    e.line(LineType::SoftOrSpace);

    if let Some(ref pathspec) = n.pathspec {
        emit_node(e, pathspec);
    }

    // PASSING clause
    if !n.passing.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PASSING_KW);
        e.space();
        // The arguments break on their own only when they don't fit on the PASSING line.
        e.group_start();
        emit_comma_separated_list(e, &n.passing, emit_node);
        e.group_end();
    }

    // RETURNING clause; JSON_EXISTS has none.
    if !is_exists && let Some(ref output) = n.output {
        e.line(LineType::SoftOrSpace);
        super::json_value_expr::emit_json_output(e, output, &mut false);
    }

    if is_query {
        // Wrapper handling (WITHOUT WRAPPER, WITH WRAPPER, etc.)
        // wrapper: 0=Undefined, 1=JswUnspec, 2=JswNone (WITHOUT), 3=JswConditional, 4=JswUnconditional
        match n.wrapper {
            2 => {
                // JswNone = WITHOUT WRAPPER
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WITHOUT_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
            }
            3 => {
                // JswConditional = WITH CONDITIONAL WRAPPER
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WITH_KW);
                e.space();
                e.token(TokenKind::CONDITIONAL_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
            }
            4 => {
                // JswUnconditional = WITH UNCONDITIONAL WRAPPER / WITH WRAPPER
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WITH_KW);
                e.space();
                e.token(TokenKind::UNCONDITIONAL_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
            }
            _ => {}
        }

        // Quote behavior (KEEP QUOTES, OMIT QUOTES)
        // quotes: 0=undefined, 1=unspec, 2=keep, 3=omit
        match n.quotes {
            2 => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::KEEP_KW);
                e.space();
                e.token(TokenKind::QUOTES_KW);
            }
            3 => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::OMIT_KW);
                e.space();
                e.token(TokenKind::QUOTES_KW);
            }
            _ => {}
        }
    }

    // ON EMPTY clause; JSON_EXISTS has none.
    if !is_exists && let Some(ref on_empty) = n.on_empty {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior(e, on_empty);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::EMPTY_KW);
    }

    // ON ERROR clause
    if let Some(ref on_error) = n.on_error {
        e.line(LineType::SoftOrSpace);
        emit_json_behavior(e, on_error);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::ERROR_KW);
    }

    e.indent_end();
    e.line(LineType::Soft);
    e.token(TokenKind::R_PAREN);

    e.group_end();
}
