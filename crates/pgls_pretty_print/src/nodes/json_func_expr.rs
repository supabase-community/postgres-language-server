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
            e.token(TokenKind::IDENT("EMPTY".to_string()));
        }
        JsonBehaviorType::JsonBehaviorEmptyArray => {
            e.token(TokenKind::IDENT("EMPTY".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ARRAY".to_string()));
        }
        JsonBehaviorType::JsonBehaviorEmptyObject => {
            e.token(TokenKind::IDENT("EMPTY".to_string()));
            e.space();
            e.token(TokenKind::IDENT("OBJECT".to_string()));
        }
        JsonBehaviorType::JsonBehaviorUnknown => {
            e.token(TokenKind::IDENT("UNKNOWN".to_string()));
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
            e.token(TokenKind::IDENT("JSON_EXISTS".to_string()));
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
            e.token(TokenKind::IDENT("JSON_QUERY".to_string()));
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
                    e.token(TokenKind::IDENT("WRAPPER".to_string()));
                }
                3 => {
                    // JswConditional = WITH CONDITIONAL WRAPPER
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::IDENT("CONDITIONAL".to_string()));
                    e.space();
                    e.token(TokenKind::IDENT("WRAPPER".to_string()));
                }
                4 => {
                    // JswUnconditional = WITH UNCONDITIONAL WRAPPER / WITH WRAPPER
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::IDENT("UNCONDITIONAL".to_string()));
                    e.space();
                    e.token(TokenKind::IDENT("WRAPPER".to_string()));
                }
                _ => {}
            }

            // Quote behavior (KEEP QUOTES, OMIT QUOTES)
            // quotes: 0=undefined, 1=unspec, 2=keep, 3=omit
            match n.quotes {
                2 => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::IDENT("KEEP".to_string()));
                    e.space();
                    e.token(TokenKind::IDENT("QUOTES".to_string()));
                }
                3 => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::IDENT("OMIT".to_string()));
                    e.space();
                    e.token(TokenKind::IDENT("QUOTES".to_string()));
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
                e.token(TokenKind::IDENT("EMPTY".to_string()));
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
            e.token(TokenKind::IDENT("JSON_VALUE".to_string()));
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
                e.token(TokenKind::IDENT("EMPTY".to_string()));
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
            e.token(TokenKind::IDENT("JSON_FUNC".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
