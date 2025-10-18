use pgt_query::NodeEnum;
use pgt_query::protobuf::DefElem;

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_def_elem(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    // Emit the option name
    if !n.defname.is_empty() {
        e.token(TokenKind::IDENT(n.defname.clone()));
    }

    // Emit the option value if present
    if let Some(ref arg) = n.arg {
        e.space();
        e.token(TokenKind::IDENT("=".to_string()));
        e.space();

        // String values in DefElem should be quoted as string literals
        if let Some(node_enum) = &arg.node {
            match node_enum {
                NodeEnum::String(s) => {
                    super::emit_string_literal(e, s);
                }
                _ => {
                    super::emit_node(arg, e);
                }
            }
        }
    }

    e.group_end();
}

/// Emit options in OPTIONS clause (e.g., for foreign data wrappers, foreign servers, COPY)
/// Syntax: name value (no equals sign)
pub(super) fn emit_options_def_elem(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    // Emit the option name
    if !n.defname.is_empty() {
        e.token(TokenKind::IDENT(n.defname.clone()));
    }

    // Emit the option value if present (no equals sign)
    if let Some(ref arg) = n.arg {
        e.space();

        // String values and booleans in OPTIONS should be quoted as string literals
        if let Some(node_enum) = &arg.node {
            match node_enum {
                NodeEnum::String(s) => {
                    super::emit_string_literal(e, s);
                }
                NodeEnum::Boolean(b) => {
                    // Boolean values in COPY/FDW options are stored as booleans
                    // but PostgreSQL parses them back as string identifiers
                    // So we emit them as lowercase identifiers (not keywords)
                    e.token(TokenKind::IDENT(if b.boolval {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }));
                }
                _ => {
                    super::emit_node(arg, e);
                }
            }
        }
    }

    e.group_end();
}

/// Emit sequence options with proper SQL syntax
/// Used by CREATE SEQUENCE and ALTER SEQUENCE
pub(super) fn emit_sequence_option(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    let defname = n.defname.as_str();

    match defname {
        "increment" => {
            e.token(TokenKind::IDENT("INCREMENT".to_string()));
            e.space();
            e.token(TokenKind::BY_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "minvalue" => {
            if let Some(ref arg) = n.arg {
                e.token(TokenKind::IDENT("MINVALUE".to_string()));
                e.space();
                super::emit_node(arg, e);
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::IDENT("MINVALUE".to_string()));
            }
        }
        "maxvalue" => {
            if let Some(ref arg) = n.arg {
                e.token(TokenKind::IDENT("MAXVALUE".to_string()));
                e.space();
                super::emit_node(arg, e);
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::IDENT("MAXVALUE".to_string()));
            }
        }
        "start" => {
            e.token(TokenKind::START_KW);
            e.space();
            e.token(TokenKind::WITH_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "restart" => {
            e.token(TokenKind::IDENT("RESTART".to_string()));
            if let Some(ref arg) = n.arg {
                e.space();
                e.token(TokenKind::WITH_KW);
                e.space();
                super::emit_node(arg, e);
            }
        }
        "cache" => {
            e.token(TokenKind::IDENT("CACHE".to_string()));
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "cycle" => {
            if n.arg.is_some() {
                // Check if the arg is a boolean/integer indicating CYCLE vs NO CYCLE
                // For now, just emit CYCLE (TODO: handle NO CYCLE)
                e.token(TokenKind::IDENT("CYCLE".to_string()));
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::IDENT("CYCLE".to_string()));
            }
        }
        "owned_by" => {
            e.token(TokenKind::IDENT("OWNED".to_string()));
            e.space();
            e.token(TokenKind::BY_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "as" => {
            e.token(TokenKind::AS_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        _ => {
            // Fallback to default behavior for unknown sequence options
            if !n.defname.is_empty() {
                e.token(TokenKind::IDENT(n.defname.clone()));
            }
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
    }

    e.group_end();
}
