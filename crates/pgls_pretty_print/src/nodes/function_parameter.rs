use pgls_query::protobuf::{FunctionParameter, FunctionParameterMode};

use super::prelude::*;

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

    // Parameter name, which the grammar spells as a function or type name
    if !fp.name.is_empty() {
        if needs_quoting_as_function_or_type_name(&fp.name) {
            emit_identifier(e, &fp.name);
        } else {
            e.token(TokenKind::IDENT(fp.name.clone()));
        }
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
        emit_node(e, defexpr);
    }
}
