use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgt_query::protobuf::AlterFdwStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_fdw_stmt(e: &mut EventEmitter, n: &AlterFdwStmt) {
    e.group_start(GroupKind::AlterFdwStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("FOREIGN".to_string()));
    e.space();
    e.token(TokenKind::IDENT("DATA".to_string()));
    e.space();
    e.token(TokenKind::IDENT("WRAPPER".to_string()));
    e.space();

    if !n.fdwname.is_empty() {
        e.token(TokenKind::IDENT(n.fdwname.clone()));
    }

    // Handler/validator functions in func_options
    if !n.func_options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        for (i, opt) in n.func_options.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }

            let def_elem = assert_node_variant!(DefElem, opt);

            match def_elem.defname.as_str() {
                "handler" => {
                    if let Some(ref arg) = def_elem.arg {
                        e.token(TokenKind::IDENT("HANDLER".to_string()));
                        e.space();
                        super::emit_node(arg, e);
                    } else {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::IDENT("HANDLER".to_string()));
                    }
                }
                "validator" => {
                    if let Some(ref arg) = def_elem.arg {
                        e.token(TokenKind::IDENT("VALIDATOR".to_string()));
                        e.space();
                        super::emit_node(arg, e);
                    } else {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::IDENT("VALIDATOR".to_string()));
                    }
                }
                _ => {
                    // Fallback for unknown options
                    super::emit_node(opt, e);
                }
            }
        }
        e.indent_end();
    }

    // OPTIONS clause
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        e.token(TokenKind::IDENT("OPTIONS".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, |n, e| {
            let def_elem = assert_node_variant!(DefElem, n);
            super::emit_options_def_elem(e, def_elem);
        });
        e.token(TokenKind::R_PAREN);
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
