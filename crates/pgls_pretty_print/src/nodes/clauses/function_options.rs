//! The options of CREATE FUNCTION and ALTER FUNCTION: `LANGUAGE sql STABLE STRICT AS $$ ... $$`.

use crate::nodes::prelude::*;

/// Returns the canonical order for a function option.
/// Postgres's canonical order (as seen in pg_dump output) is:
/// 1. LANGUAGE
/// 2. TRANSFORM
/// 3. WINDOW
/// 4. IMMUTABLE / STABLE / VOLATILE (volatility)
/// 5. LEAKPROOF / NOT LEAKPROOF
/// 6. STRICT / CALLED ON NULL INPUT (strict)
/// 7. SECURITY DEFINER / SECURITY INVOKER (security)
/// 8. PARALLEL (parallel)
/// 9. COST (cost)
/// 10. ROWS (rows)
/// 11. SUPPORT (support)
/// 12. SET options (set)
/// 13. AS (function body)
/// Where an option of CREATE FUNCTION or ALTER FUNCTION goes: the printer writes them in this
/// order, and the AST comparison sorts both sides by it.
pub(crate) fn function_option_order(defname: &str) -> usize {
    match defname.to_lowercase().as_str() {
        "language" => 0,
        "transform" => 1,
        "window" => 2,
        "volatility" => 3,
        "leakproof" => 4,
        "strict" => 5,
        "security" => 6,
        "parallel" => 7,
        "cost" => 8,
        "rows" => 9,
        "support" => 10,
        "set" => 11,
        "as" => 12,
        _ => 100, // Unknown options go last
    }
}

/// Sort function options according to Postgres's canonical order.
pub(crate) fn sort_function_options(
    options: &[pgls_query::protobuf::Node],
) -> Vec<pgls_query::protobuf::Node> {
    let mut sorted: Vec<pgls_query::protobuf::Node> = options.to_vec();
    sorted.sort_by_key(|node| {
        if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = &node.node {
            function_option_order(&def_elem.defname)
        } else {
            100 // Non-DefElem nodes go last
        }
    });
    sorted
}

pub(crate) fn format_function_option(e: &mut EventEmitter, d: &pgls_query::protobuf::DefElem) {
    let defname_lower = d.defname.to_lowercase();

    match defname_lower.as_str() {
        "as" => {
            e.token(TokenKind::AS_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                // AS can have a list (for C functions with library and symbol)
                // or a single string (for SQL/plpgsql functions)
                if let Some(pgls_query::NodeEnum::List(list)) = &arg.node {
                    if list.items.len() == 1 {
                        // Single item: either library name (C) or SQL body (SQL/plpgsql)
                        if let Some(pgls_query::NodeEnum::String(s)) = &list.items[0].node {
                            // Use dollar quotes for function bodies (they handle embedded quotes better)
                            emit_body_literal(e, &s.sval);
                        } else {
                            emit_node(e, &list.items[0]);
                        }
                    } else if list.items.len() == 2 {
                        // Two items: library and symbol for C functions
                        // C functions use regular string literals for library paths
                        if let Some(pgls_query::NodeEnum::String(s)) = &list.items[0].node {
                            emit_string_literal(e, s);
                        } else {
                            emit_node(e, &list.items[0]);
                        }
                        e.token(TokenKind::COMMA);
                        e.space();
                        if let Some(pgls_query::NodeEnum::String(s)) = &list.items[1].node {
                            emit_string_literal(e, s);
                        } else {
                            emit_node(e, &list.items[1]);
                        }
                    } else {
                        // Fallback: emit the list as-is
                        emit_node(e, arg);
                    }
                } else {
                    emit_node(e, arg);
                }
            }
        }
        "language" => {
            e.token(TokenKind::LANGUAGE_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::String(s)) = &arg.node {
                    // Language names like 'sql', 'plpgsql', 'c' don't need quotes
                    emit_identifier_maybe_quoted(e, &s.sval);
                } else {
                    emit_node(e, arg);
                }
            }
        }
        "volatility" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::String(s)) = &arg.node {
                    let volatility = s.sval.to_uppercase();
                    match volatility.as_str() {
                        "IMMUTABLE" => e.token(TokenKind::IMMUTABLE_KW),
                        "STABLE" => e.token(TokenKind::STABLE_KW),
                        "VOLATILE" => e.token(TokenKind::VOLATILE_KW),
                        _ => emit_keyword(e, &s.sval),
                    }
                } else {
                    emit_node(e, arg);
                }
            }
        }
        "strict" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::STRICT_KW);
                    } else {
                        e.token(TokenKind::CALLED_KW);
                        e.space();
                        e.token(TokenKind::ON_KW);
                        e.space();
                        e.token(TokenKind::NULL);
                        e.space();
                        e.token(TokenKind::INPUT_KW);
                    }
                } else {
                    e.token(TokenKind::STRICT_KW);
                }
            } else {
                e.token(TokenKind::STRICT_KW);
            }
        }
        "security" => {
            e.token(TokenKind::SECURITY_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::DEFINER_KW);
                    } else {
                        e.token(TokenKind::INVOKER_KW);
                    }
                } else {
                    emit_node(e, arg);
                }
            }
        }
        "leakproof" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::LEAKPROOF_KW);
                    } else {
                        e.token(TokenKind::NOT_KW);
                        e.space();
                        e.token(TokenKind::LEAKPROOF_KW);
                    }
                } else {
                    e.token(TokenKind::LEAKPROOF_KW);
                }
            } else {
                e.token(TokenKind::LEAKPROOF_KW);
            }
        }
        "parallel" => {
            e.token(TokenKind::PARALLEL_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                if let Some(pgls_query::NodeEnum::String(s)) = &arg.node {
                    emit_keyword(e, &s.sval);
                } else {
                    emit_node(e, arg);
                }
            }
        }
        "cost" => {
            e.token(TokenKind::COST_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                emit_node(e, arg);
            }
        }
        "rows" => {
            e.token(TokenKind::ROWS_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                emit_node(e, arg);
            }
        }
        "support" => {
            e.token(TokenKind::IDENT("support".to_string()));
            e.space();
            if let Some(ref arg) = d.arg {
                emit_node(e, arg);
            }
        }
        "set" => {
            // The arg is a VariableSetStmt which will emit SET/RESET itself
            if let Some(ref arg) = d.arg {
                emit_node(e, arg);
            }
        }
        "transform" => {
            // TRANSFORM FOR TYPE a, FOR TYPE b: the arg is the list of type names
            e.token(TokenKind::TRANSFORM_KW);
            e.space();
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::List(list)) = &arg.node
            {
                e.group_start();
                e.indent_start();
                for (index, item) in list.items.iter().enumerate() {
                    if index > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    e.token(TokenKind::FOR_KW);
                    e.space();
                    e.token(TokenKind::TYPE_KW);
                    e.space();
                    emit_node(e, item);
                }
                e.indent_end();
                e.group_end();
            }
        }
        "window" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
                && b.boolval
            {
                e.token(TokenKind::WINDOW_KW);
            }
        }
        _ => {
            // Default: emit option name and value
            emit_keyword(e, &d.defname);
            if let Some(ref arg) = d.arg {
                e.space();
                emit_node(e, arg);
            }
        }
    }
}
