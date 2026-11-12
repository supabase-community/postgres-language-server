use pgls_query::protobuf::AlterFdwStmt;

use super::prelude::*;

pub(super) fn emit_alter_fdw_stmt(e: &mut EventEmitter, n: &AlterFdwStmt) {
    e.group_start(GroupKind::AlterFdwStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::FOREIGN_KW);
    e.space();
    e.token(TokenKind::DATA_KW);
    e.space();
    e.token(TokenKind::WRAPPER_KW);
    e.space();

    if !n.fdwname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.fdwname);
    }

    // Handler/validator functions in func_options - space-separated, not comma-separated
    if !n.func_options.is_empty() {
        e.indent_start();
        for opt in n.func_options.iter() {
            e.line(LineType::SoftOrSpace);
            let def_elem = assert_node_variant!(DefElem, opt);

            match def_elem.defname.as_str() {
                "handler" => {
                    if let Some(ref arg) = def_elem.arg {
                        e.token(TokenKind::HANDLER_KW);
                        e.space();
                        emit_node(arg, e);
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
                        emit_node(arg, e);
                    } else {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::VALIDATOR_KW);
                    }
                }
                _ => {
                    // Fallback for unknown options
                    emit_node(opt, e);
                }
            }
        }
        e.indent_end();
    }

    // OPTIONS clause
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        e.token(TokenKind::OPTIONS_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.indent_end();
    }

    e.group_end();
}
