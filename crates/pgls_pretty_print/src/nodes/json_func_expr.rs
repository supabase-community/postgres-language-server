use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::{JsonBehavior, JsonBehaviorType, JsonFuncExpr};

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
                super::emit_node(expr, e);
            }
        }
        _ => {}
    }
}

pub(super) fn emit_json_func_expr(e: &mut EventEmitter, n: &JsonFuncExpr) {
    e.group_start(GroupKind::JsonFuncExpr);

    // Map JSON function operation types
    // 0=Undefined, 1=JsonExistsOp, 2=JsonQueryOp, 3=JsonValueOp, 4=JsonTableOp
    match n.op {
        1 => {
            // JSON_EXISTS
            e.token(TokenKind::JSON_EXISTS_KW);
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
                if let Some(ref format) = context.format {
                    super::json_value_expr::emit_json_format(e, format);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            // PASSING clause
            if !n.passing.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::PASSING_KW);
                e.space();
                super::node_list::emit_comma_separated_list(e, &n.passing, super::emit_node);
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

            e.token(TokenKind::R_PAREN);
        }
        2 => {
            // JSON_QUERY
            e.token(TokenKind::JSON_QUERY_KW);
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
                if let Some(ref format) = context.format {
                    super::json_value_expr::emit_json_format(e, format);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            // PASSING clause
            if !n.passing.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::PASSING_KW);
                e.space();
                super::node_list::emit_comma_separated_list(e, &n.passing, super::emit_node);
            }

            // RETURNING clause
            if let Some(ref output) = n.output {
                let mut has_content = true;
                e.line(LineType::SoftOrSpace);
                super::json_value_expr::emit_json_output(e, output, &mut has_content);
            }

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

            // ON EMPTY clause
            if let Some(ref on_empty) = n.on_empty {
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

            e.token(TokenKind::R_PAREN);
        }
        3 => {
            // JSON_VALUE
            e.token(TokenKind::JSON_VALUE_KW);
            e.token(TokenKind::L_PAREN);

            if let Some(ref context) = n.context_item {
                if let Some(ref raw_expr) = context.raw_expr {
                    super::emit_node(raw_expr, e);
                }
                if let Some(ref format) = context.format {
                    super::json_value_expr::emit_json_format(e, format);
                }
            }

            e.token(TokenKind::COMMA);
            e.space();

            if let Some(ref pathspec) = n.pathspec {
                super::emit_node(pathspec, e);
            }

            // PASSING clause
            if !n.passing.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::PASSING_KW);
                e.space();
                super::node_list::emit_comma_separated_list(e, &n.passing, super::emit_node);
            }

            // RETURNING clause
            if let Some(ref output) = n.output {
                let mut has_content = true;
                e.line(LineType::SoftOrSpace);
                super::json_value_expr::emit_json_output(e, output, &mut has_content);
            }

            // ON EMPTY clause
            if let Some(ref on_empty) = n.on_empty {
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

            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Unknown JSON function - emit placeholder
            e.token(TokenKind::IDENT("json_func".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
