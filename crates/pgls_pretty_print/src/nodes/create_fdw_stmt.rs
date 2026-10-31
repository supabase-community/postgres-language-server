use pgls_query::protobuf::CreateFdwStmt;

use super::prelude::*;

pub(super) fn emit_create_fdw_stmt(e: &mut EventEmitter, n: &CreateFdwStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::DATA_KW);
    e.space();
    e.token(TokenKind::WRAPPER_KW);
    e.space();
    emit_identifier_maybe_quoted(e, &n.fdwname);

    // Handler and validator functions
    if !n.func_options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        for opt in &n.func_options {
            let def_elem = assert_node_variant!(DefElem, opt);

            match def_elem.defname.as_str() {
                "handler" => {
                    if let Some(ref arg) = def_elem.arg {
                        e.token(TokenKind::HANDLER_KW);
                        e.space();
                        emit_node(e, arg);
                    } else {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::HANDLER_KW);
                    }
                }
                "validator" => {
                    if let Some(ref arg) = def_elem.arg {
                        e.token(TokenKind::VALIDATOR_KW);
                        e.space();
                        emit_node(e, arg);
                    } else {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::VALIDATOR_KW);
                    }
                }
                _ => {
                    // Fallback for unknown options
                    emit_node(e, opt);
                }
            }
            e.space();
        }
        e.indent_end();
    }

    // Options
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |e, n| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.indent_end();
    }

    e.group_end();
}
