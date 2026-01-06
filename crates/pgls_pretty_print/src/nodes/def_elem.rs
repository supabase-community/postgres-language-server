use pgls_query::NodeEnum;
use pgls_query::protobuf::{DefElem, DefElemAction};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_def_elem(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    // Emit namespace if present (namespace.name syntax)
    if !n.defnamespace.is_empty() {
        super::string::emit_identifier_maybe_quoted(e, &n.defnamespace);
        e.token(TokenKind::DOT);
    }

    // Emit the option name - use maybe_quoted to preserve case for mixed-case identifiers
    if !n.defname.is_empty() {
        super::string::emit_identifier_maybe_quoted(e, &n.defname);
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
/// Syntax: [SET|ADD|DROP] name value (no equals sign)
/// For ALTER statements, defaction indicates SET/ADD/DROP prefix
pub(super) fn emit_options_def_elem(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    // Emit SET/ADD/DROP prefix for ALTER statements
    // defaction: 1=DefelemUnspec, 2=DefelemSet, 3=DefelemAdd, 4=DefelemDrop
    match n.defaction() {
        DefElemAction::DefelemSet => {
            e.token(TokenKind::SET_KW);
            e.space();
        }
        DefElemAction::DefelemAdd => {
            e.token(TokenKind::ADD_KW);
            e.space();
        }
        DefElemAction::DefelemDrop => {
            e.token(TokenKind::DROP_KW);
            e.space();
        }
        _ => {}
    }

    // Emit the option name - use maybe_quoted to preserve case for mixed-case identifiers
    if !n.defname.is_empty() {
        super::string::emit_identifier_maybe_quoted(e, &n.defname);
    }

    // For DROP, there's no value
    if n.defaction() == DefElemAction::DefelemDrop {
        e.group_end();
        return;
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
                    // Boolean values in COPY/FDW options need to use TRUE/FALSE keywords
                    // Using lowercase 'true' would parse back as a string identifier
                    if b.boolval {
                        e.token(TokenKind::TRUE_KW);
                    } else {
                        e.token(TokenKind::FALSE_KW);
                    }
                }
                _ => {
                    super::emit_node(arg, e);
                }
            }
        }
    }

    e.group_end();
}

/// Emit role options for GRANT...WITH syntax
/// Uses `NAME TRUE` / `NAME FALSE` syntax for INHERIT, SET, ADMIN
pub(super) fn emit_grant_role_option(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    let defname = n.defname.as_str();

    // Check if arg is a boolean
    let is_true = n.arg.as_ref().and_then(|arg| {
        if let Some(NodeEnum::Boolean(b)) = &arg.node {
            Some(b.boolval)
        } else if let Some(NodeEnum::Integer(i)) = &arg.node {
            Some(i.ival != 0)
        } else {
            None
        }
    });

    // Emit the option name in uppercase
    let opt_name = match defname {
        "admin" => "ADMIN",
        "inherit" => "INHERIT",
        "set" => "SET",
        _ => &defname.to_uppercase(),
    };
    e.token(TokenKind::IDENT(opt_name.to_string()));

    // Emit TRUE or FALSE
    if let Some(val) = is_true {
        e.space();
        if val {
            e.token(TokenKind::TRUE_KW);
        } else {
            e.token(TokenKind::FALSE_KW);
        }
    }

    e.group_end();
}

/// Emit role options with proper SQL syntax
/// Used by CREATE ROLE, ALTER ROLE, CREATE USER, etc.
/// Role options like SUPERUSER, NOSUPERUSER, etc. use special syntax
pub(super) fn emit_role_option(e: &mut EventEmitter, n: &DefElem) {
    e.group_start(GroupKind::DefElem);

    let defname = n.defname.as_str();

    // Check if arg is a boolean
    let is_true = n.arg.as_ref().and_then(|arg| {
        if let Some(NodeEnum::Boolean(b)) = &arg.node {
            Some(b.boolval)
        } else if let Some(NodeEnum::Integer(i)) = &arg.node {
            Some(i.ival != 0)
        } else {
            None
        }
    });

    match defname {
        "superuser" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("superuser".to_string()));
            } else {
                e.token(TokenKind::IDENT("nosuperuser".to_string()));
            }
        }
        "createdb" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("createdb".to_string()));
            } else {
                e.token(TokenKind::IDENT("nocreatedb".to_string()));
            }
        }
        "createrole" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("createrole".to_string()));
            } else {
                e.token(TokenKind::IDENT("nocreaterole".to_string()));
            }
        }
        "inherit" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::INHERIT_KW);
            } else {
                e.token(TokenKind::IDENT("noinherit".to_string()));
            }
        }
        "canlogin" | "login" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("login".to_string()));
            } else {
                e.token(TokenKind::IDENT("nologin".to_string()));
            }
        }
        "isreplication" | "replication" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("replication".to_string()));
            } else {
                e.token(TokenKind::IDENT("noreplication".to_string()));
            }
        }
        "bypassrls" => {
            if is_true.unwrap_or(true) {
                e.token(TokenKind::IDENT("bypassrls".to_string()));
            } else {
                e.token(TokenKind::IDENT("nobypassrls".to_string()));
            }
        }
        "connectionlimit" => {
            e.token(TokenKind::CONNECTION_KW);
            e.space();
            e.token(TokenKind::LIMIT_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "password" => {
            if n.arg.is_none() {
                e.token(TokenKind::PASSWORD_KW);
                e.space();
                e.token(TokenKind::NULL_KW);
            } else {
                e.token(TokenKind::PASSWORD_KW);
                if let Some(ref arg) = n.arg {
                    e.space();
                    // Password values must be single-quoted strings
                    if let Some(NodeEnum::String(s)) = &arg.node {
                        super::emit_string_literal(e, s);
                    } else {
                        super::emit_node(arg, e);
                    }
                }
            }
        }
        "encryptedPassword" | "unencryptedPassword" => {
            e.token(TokenKind::PASSWORD_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                // Password values must be single-quoted strings
                if let Some(NodeEnum::String(s)) = &arg.node {
                    super::emit_string_literal(e, s);
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "validUntil" => {
            e.token(TokenKind::VALID_KW);
            e.space();
            e.token(TokenKind::UNTIL_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                // VALID UNTIL values must be single-quoted strings
                if let Some(NodeEnum::String(s)) = &arg.node {
                    super::emit_string_literal(e, s);
                } else {
                    super::emit_node(arg, e);
                }
            }
        }
        "rolemembers" => {
            e.token(TokenKind::ROLE_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "adminmembers" => {
            e.token(TokenKind::ADMIN_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "addroleto" => {
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::ROLE_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "sysid" => {
            e.token(TokenKind::SYSID_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        _ => {
            // Fallback for unknown role options
            if !n.defname.is_empty() {
                e.token(TokenKind::IDENT(n.defname.to_uppercase()));
            }
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
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
            e.token(TokenKind::INCREMENT_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "minvalue" => {
            if let Some(ref arg) = n.arg {
                e.token(TokenKind::MINVALUE_KW);
                e.space();
                super::emit_node(arg, e);
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::MINVALUE_KW);
            }
        }
        "maxvalue" => {
            if let Some(ref arg) = n.arg {
                e.token(TokenKind::MAXVALUE_KW);
                e.space();
                super::emit_node(arg, e);
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::MAXVALUE_KW);
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
            e.token(TokenKind::RESTART_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                e.token(TokenKind::WITH_KW);
                e.space();
                super::emit_node(arg, e);
            }
        }
        "cache" => {
            e.token(TokenKind::CACHE_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
        "cycle" => {
            if n.arg.is_some() {
                // Check if the arg is a boolean/integer indicating CYCLE vs NO CYCLE
                // For now, just emit CYCLE (TODO: handle NO CYCLE)
                e.token(TokenKind::CYCLE_KW);
            } else {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::CYCLE_KW);
            }
        }
        "owned_by" => {
            e.token(TokenKind::OWNED_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            if let Some(ref arg) = n.arg {
                e.space();
                // The arg is a list of identifiers that should be dot-separated (table.column)
                if let Some(pgls_query::NodeEnum::List(list)) = arg.node.as_ref() {
                    super::node_list::emit_dot_separated_list(e, &list.items);
                } else {
                    super::emit_node(arg, e);
                }
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
