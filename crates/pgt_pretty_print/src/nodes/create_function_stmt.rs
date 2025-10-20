use pgt_query::protobuf::{CreateFunctionStmt, FunctionParameter, FunctionParameterMode};

use super::node_list::emit_dot_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_create_function_stmt(e: &mut EventEmitter, n: &CreateFunctionStmt) {
    e.group_start(GroupKind::CreateFunctionStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::REPLACE_KW);
        e.space();
    }

    if n.is_procedure {
        e.token(TokenKind::PROCEDURE_KW);
    } else {
        e.token(TokenKind::FUNCTION_KW);
    }

    e.space();

    // Function name (qualified name)
    emit_dot_separated_list(e, &n.funcname);

    // Parameters
    e.token(TokenKind::L_PAREN);
    if !n.parameters.is_empty() {
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.parameters, |param, e| {
            if let Some(pgt_query::NodeEnum::FunctionParameter(fp)) = &param.node {
                emit_function_parameter(e, fp);
            }
        });
        e.indent_end();
        e.line(LineType::Soft);
    }
    e.token(TokenKind::R_PAREN);

    // Return type (only for functions, not procedures)
    if !n.is_procedure {
        if let Some(ref return_type) = n.return_type {
            e.space();
            e.token(TokenKind::RETURNS_KW);
            e.space();
            super::emit_type_name(e, return_type);
        }
    }

    // Options
    for option in &n.options {
        if let Some(pgt_query::NodeEnum::DefElem(def_elem)) = &option.node {
            e.space();
            format_function_option(e, def_elem);
        }
    }

    // SQL body (if present, modern syntax)
    if let Some(ref sql_body) = n.sql_body {
        e.space();
        e.token(TokenKind::BEGIN_KW);
        e.space();
        e.token(TokenKind::ATOMIC_KW);
        e.indent_start();
        e.line(LineType::Hard);
        super::emit_node(sql_body, e);
        e.indent_end();
        e.line(LineType::Hard);
        e.token(TokenKind::END_KW);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

pub(super) fn emit_function_parameter(e: &mut EventEmitter, fp: &FunctionParameter) {
    // Parameter mode (IN, OUT, INOUT, VARIADIC)
    match fp.mode() {
        FunctionParameterMode::FuncParamIn => {
            e.token(TokenKind::IN_KW);
            e.space();
        }
        FunctionParameterMode::FuncParamOut => {
            e.token(TokenKind::OUT_KW);
            e.space();
        }
        FunctionParameterMode::FuncParamInout => {
            e.token(TokenKind::INOUT_KW);
            e.space();
        }
        FunctionParameterMode::FuncParamVariadic => {
            e.token(TokenKind::VARIADIC_KW);
            e.space();
        }
        FunctionParameterMode::FuncParamTable => {
            // TABLE mode is not emitted as a prefix
        }
        FunctionParameterMode::FuncParamDefault => {
            // Default mode doesn't emit anything
        }
        FunctionParameterMode::Undefined => {}
    }

    // Parameter name
    if !fp.name.is_empty() {
        super::emit_identifier(e, &fp.name);
        e.space();
    }

    // Parameter type
    if let Some(ref arg_type) = fp.arg_type {
        super::emit_type_name(e, arg_type);
    }

    // Default value
    if let Some(ref defexpr) = fp.defexpr {
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        super::emit_node(defexpr, e);
    }
}

pub(super) fn format_function_option(e: &mut EventEmitter, d: &pgt_query::protobuf::DefElem) {
    let defname_lower = d.defname.to_lowercase();

    match defname_lower.as_str() {
        "as" => {
            e.token(TokenKind::AS_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                // AS can have a list (for C functions with library and symbol)
                // or a single string (for SQL/plpgsql functions)
                if let Some(pgt_query::NodeEnum::List(list)) = &arg.node {
                    if list.items.len() == 1 {
                        // Single item: either library name (C) or SQL body (SQL/plpgsql)
                        if let Some(pgt_query::NodeEnum::String(s)) = &list.items[0].node {
                            super::emit_string_literal(e, s);
                        } else {
                            super::emit_node(&list.items[0], e);
                        }
                    } else if list.items.len() == 2 {
                        // Two items: library and symbol for C functions
                        if let Some(pgt_query::NodeEnum::String(s)) = &list.items[0].node {
                            super::emit_string_literal(e, s);
                        } else {
                            super::emit_node(&list.items[0], e);
                        }
                        e.token(TokenKind::COMMA);
                        e.space();
                        if let Some(pgt_query::NodeEnum::String(s)) = &list.items[1].node {
                            super::emit_string_literal(e, s);
                        } else {
                            super::emit_node(&list.items[1], e);
                        }
                    } else {
                        // Fallback: emit the list as-is
                        super::emit_node(arg, e);
                    }
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "language" => {
            e.token(TokenKind::LANGUAGE_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::String(s)) = &arg.node {
                    super::emit_identifier(e, &s.sval);
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "volatility" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::String(s)) = &arg.node {
                    let volatility = s.sval.to_uppercase();
                    match volatility.as_str() {
                        "IMMUTABLE" => e.token(TokenKind::IMMUTABLE_KW),
                        "STABLE" => e.token(TokenKind::STABLE_KW),
                        "VOLATILE" => e.token(TokenKind::VOLATILE_KW),
                        _ => e.token(TokenKind::IDENT(volatility)),
                    }
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "strict" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("STRICT".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("CALLED ON NULL INPUT".to_string()));
                    }
                } else {
                    e.token(TokenKind::IDENT("STRICT".to_string()));
                }
            } else {
                e.token(TokenKind::IDENT("STRICT".to_string()));
            }
        }
        "security" => {
            e.token(TokenKind::SECURITY_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("DEFINER".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("INVOKER".to_string()));
                    }
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "leakproof" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::LEAKPROOF_KW);
                    } else {
                        e.token(TokenKind::IDENT("NOT LEAKPROOF".to_string()));
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
                if let Some(pgt_query::NodeEnum::String(s)) = &arg.node {
                    let parallel = s.sval.to_uppercase();
                    e.token(TokenKind::IDENT(parallel));
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "cost" => {
            e.token(TokenKind::COST_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                super::emit_node(arg, e);
            }
        }
        "rows" => {
            e.token(TokenKind::ROWS_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                super::emit_node(arg, e);
            }
        }
        "support" => {
            e.token(TokenKind::IDENT("SUPPORT".to_string()));
            e.space();
            if let Some(ref arg) = d.arg {
                super::emit_node(arg, e);
            }
        }
        "set" => {
            e.token(TokenKind::SET_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                super::emit_node(arg, e);
            }
        }
        "window" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::WINDOW_KW);
                    }
                }
            }
        }
        _ => {
            // Default: emit option name and value
            let defname_upper = d.defname.to_uppercase();
            e.token(TokenKind::IDENT(defname_upper));
            if let Some(ref arg) = d.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
    }
}
