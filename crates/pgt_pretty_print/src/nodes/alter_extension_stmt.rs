use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterExtensionStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_extension_stmt(e: &mut EventEmitter, n: &AlterExtensionStmt) {
    e.group_start(GroupKind::AlterExtensionStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("EXTENSION".to_string()));
    e.space();

    if !n.extname.is_empty() {
        e.token(TokenKind::IDENT(n.extname.clone()));
    }

    if !n.options.is_empty() {
        e.space();
        // ALTER EXTENSION has special syntax for UPDATE TO version
        // Check if options contain "new_version" - if so, emit UPDATE TO syntax
        let has_update_to = n.options.iter().any(|opt| {
            if let Some(pgt_query::NodeEnum::DefElem(d)) = &opt.node {
                d.defname == "new_version"
            } else {
                false
            }
        });

        if has_update_to {
            // Find the new_version option and emit UPDATE TO syntax
            for opt in &n.options {
                if let Some(pgt_query::NodeEnum::DefElem(d)) = &opt.node {
                    if d.defname == "new_version" {
                        e.token(TokenKind::UPDATE_KW);
                        e.space();
                        e.token(TokenKind::TO_KW);
                        if let Some(ref arg) = d.arg {
                            e.space();
                            // Version must be a string literal (quoted)
                            if let Some(pgt_query::NodeEnum::String(s)) = &arg.node {
                                super::emit_string_literal(e, s);
                            } else {
                                super::emit_node(arg, e);
                            }
                        }
                    }
                }
            }
        } else {
            // For other options, emit as comma-separated list
            emit_comma_separated_list(e, &n.options, super::emit_node);
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
