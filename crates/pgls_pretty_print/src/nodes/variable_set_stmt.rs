use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::{
    NodeEnum,
    protobuf::{Node, VariableSetStmt},
};

/// Emit a SET statement argument
/// Special handling: AConst with string values should be emitted as unquoted identifiers
fn emit_set_arg(node: &Node, e: &mut EventEmitter) {
    if let Some(NodeEnum::AConst(a_const)) = &node.node {
        if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) = &a_const.val {
            // Check if this looks like it should be an identifier (not a quoted string)
            // In SET statements, simple identifiers like schema names are stored as string constants
            // but should be emitted without quotes
            let val = &s.sval;

            // Emit as identifier (no quotes) if it looks like a simple identifier
            // This includes schema names, role names, etc.
            if is_simple_identifier(val) {
                e.group_start(GroupKind::String);
                e.token(TokenKind::IDENT(val.clone()));
                e.group_end();
                return;
            }
        }
    }

    // For all other cases (numbers, actual string literals with special chars, etc.),
    // use the normal emission
    super::emit_node(node, e);
}

/// Check if a string should be emitted as a simple unquoted identifier
fn is_simple_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let first_char = s.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest must be alphanumeric, underscore, or dollar sign
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        return false;
    }

    // Check for reserved keywords that must be quoted
    // These are common SQL keywords that can't be used as unquoted values
    let reserved_keywords = [
        "all", "and", "as", "asc", "between", "by", "case", "check", "create", "default", "delete",
        "desc", "distinct", "drop", "else", "end", "false", "for", "from", "full", "group",
        "having", "if", "in", "index", "inner", "insert", "into", "is", "join", "left", "like",
        "limit", "not", "null", "offset", "on", "or", "order", "outer", "primary", "right",
        "select", "set", "table", "then", "to", "true", "union", "unique", "update", "using",
        "values", "when", "where", "with", "any", "some", "none", "only",
    ];

    !reserved_keywords.contains(&s.to_lowercase().as_str())
}

/// Emit a dot-separated variable name, quoting parts that need it
fn emit_variable_name(e: &mut EventEmitter, name: &str) {
    let parts: Vec<&str> = name.split('.').collect();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        if is_simple_identifier(part) {
            e.token(TokenKind::IDENT(part.to_string()));
        } else {
            // Quote the identifier
            e.token(TokenKind::IDENT(format!("\"{part}\"")));
        }
    }
}

/// Emit the SET/RESET statement body without a semicolon (for use in ALTER ROLE SET, etc.)
pub(super) fn emit_variable_set_stmt_no_semicolon(e: &mut EventEmitter, n: &VariableSetStmt) {
    e.group_start(GroupKind::VariableSetStmt);
    emit_variable_set_stmt_inner(e, n);
    e.group_end();
}

pub(super) fn emit_variable_set_stmt(e: &mut EventEmitter, n: &VariableSetStmt) {
    e.group_start(GroupKind::VariableSetStmt);
    emit_variable_set_stmt_inner(e, n);
    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_variable_set_stmt_inner(e: &mut EventEmitter, n: &VariableSetStmt) {
    // Handle different kinds of SET statements
    // kind 1 = VAR_SET_VALUE (most common)
    // kind 2 = VAR_SET_DEFAULT
    // kind 3 = VAR_SET_CURRENT
    // kind 4 = VAR_SET_MULTI
    // kind 5 = VAR_RESET
    // kind 6 = VAR_RESET_ALL

    if n.kind == 5 {
        // VAR_RESET - emit RESET variable_name
        e.token(TokenKind::RESET_KW);
        e.space();
        emit_variable_name(e, &n.name);
        return;
    } else if n.kind == 6 {
        // VAR_RESET_ALL - emit RESET ALL
        e.token(TokenKind::RESET_KW);
        e.space();
        e.token(TokenKind::ALL_KW);
        return;
    }

    // Emit SET keyword for other variants
    e.token(TokenKind::SET_KW);

    // Emit LOCAL if applicable
    if n.is_local {
        e.space();
        e.token(TokenKind::LOCAL_KW);
    }

    e.space();

    if n.kind == 1 {
        // Handle special PostgreSQL SET variants
        match n.name.to_lowercase().as_str() {
            "timezone" => {
                e.token(TokenKind::TIME_KW);
                e.space();
                e.token(TokenKind::ZONE_KW);
            }
            "catalog" => {
                e.token(TokenKind::CATALOG_KW);
            }
            "names" => {
                // SET NAMES is parsed with name="names"
                e.token(TokenKind::NAMES_KW);
            }
            "client_encoding" => {
                // SET client_encoding should stay as client_encoding
                e.token(TokenKind::IDENT(n.name.clone()));
            }
            "role" => {
                e.token(TokenKind::ROLE_KW);
            }
            "session_authorization" => {
                e.token(TokenKind::SESSION_KW);
                e.space();
                e.token(TokenKind::AUTHORIZATION_KW);
            }
            "transaction_isolation" => {
                e.token(TokenKind::TRANSACTION_KW);
                e.space();
                e.token(TokenKind::ISOLATION_KW);
                e.space();
                e.token(TokenKind::LEVEL_KW);
            }
            _ => {
                // Generic variable name - may contain dots and special chars
                emit_variable_name(e, &n.name);
            }
        }

        // Emit value assignment
        if !n.args.is_empty() {
            // Determine whether to use = or TO or nothing
            // SESSION AUTHORIZATION uses no connector (just space)
            // SET TIME ZONE uses no connector (just space before value)
            // Most special variables use TO
            // Generic variables use =
            let uses_to = matches!(
                n.name.to_lowercase().as_str(),
                "catalog"
                    | "names"
                    | "client_encoding"
                    | "role"
                    | "transaction_isolation"
                    | "search_path"
            );

            let no_connector = matches!(
                n.name.to_lowercase().as_str(),
                "session_authorization" | "timezone"
            );

            e.space();
            if !no_connector {
                if uses_to {
                    e.token(TokenKind::TO_KW);
                } else {
                    e.token(TokenKind::IDENT("=".to_string()));
                }
                e.space();
            }
            // For SET statements, string constants should be emitted as identifiers (no quotes)
            // unless they're actual quoted strings in the original
            emit_comma_separated_list(e, &n.args, emit_set_arg);
        }
    } else if n.kind == 2 {
        // VAR_SET_DEFAULT
        // Special case: SET SESSION AUTHORIZATION DEFAULT (no TO keyword)
        if n.name.to_lowercase() == "session_authorization" {
            e.token(TokenKind::SESSION_KW);
            e.space();
            e.token(TokenKind::AUTHORIZATION_KW);
            e.space();
            e.token(TokenKind::DEFAULT_KW);
        } else {
            // Determine whether to use = or TO
            let uses_to = matches!(
                n.name.to_lowercase().as_str(),
                "timezone"
                    | "catalog"
                    | "names"
                    | "client_encoding"
                    | "role"
                    | "transaction_isolation"
                    | "search_path"
            );

            emit_variable_name(e, &n.name);
            e.space();
            if uses_to {
                e.token(TokenKind::TO_KW);
            } else {
                e.token(TokenKind::IDENT("=".to_string()));
            }
            e.space();
            e.token(TokenKind::DEFAULT_KW);
        }
    } else if n.kind == 3 {
        // VAR_SET_CURRENT
        emit_variable_name(e, &n.name);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::CURRENT_KW);
    } else if n.kind == 4 {
        // VAR_SET_MULTI - used for SET TRANSACTION/SESSION CHARACTERISTICS
        // name = "TRANSACTION" or "SESSION CHARACTERISTICS" or "TRANSACTION SNAPSHOT"
        // args = list of DefElems with transaction options, or a string for SNAPSHOT
        if n.name.to_uppercase() == "SESSION CHARACTERISTICS" {
            // Special syntax: SET SESSION CHARACTERISTICS AS TRANSACTION ...
            e.token(TokenKind::SESSION_KW);
            e.space();
            e.token(TokenKind::IDENT("CHARACTERISTICS".to_string()));
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::TRANSACTION_KW);

            // Emit transaction options from args (DefElems) with soft line breaks
            for arg in &n.args {
                if let Some(NodeEnum::DefElem(def)) = &arg.node {
                    e.line(crate::emitter::LineType::SoftOrSpace);
                    emit_transaction_option(e, def);
                }
            }
        } else if n.name.to_uppercase() == "TRANSACTION SNAPSHOT" {
            // SET TRANSACTION SNAPSHOT 'snapshot_id'
            e.token(TokenKind::TRANSACTION_KW);
            e.space();
            e.token(TokenKind::IDENT("SNAPSHOT".to_string()));
            e.space();
            // The snapshot id is in args as a string
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
        } else {
            emit_variable_name(e, &n.name);

            // Emit transaction options from args (DefElems) with soft line breaks
            for arg in &n.args {
                if let Some(NodeEnum::DefElem(def)) = &arg.node {
                    e.line(crate::emitter::LineType::SoftOrSpace);
                    emit_transaction_option(e, def);
                }
            }
        }
    } else {
        // Other variants
        emit_variable_name(e, &n.name);
    }
}

/// Emit a transaction option from a DefElem (used in SET TRANSACTION)
fn emit_transaction_option(e: &mut EventEmitter, def: &pgls_query::protobuf::DefElem) {
    match def.defname.as_str() {
        "transaction_isolation" => {
            e.token(TokenKind::ISOLATION_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);
            e.space();
            // Value is the isolation level name
            if let Some(ref arg) = def.arg {
                if let Some(NodeEnum::AConst(a_const)) = &arg.node {
                    if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) = &a_const.val {
                        // Emit the isolation level as uppercase keywords
                        let level = s.sval.to_uppercase();
                        match level.as_str() {
                            "SERIALIZABLE" => {
                                e.token(TokenKind::SERIALIZABLE_KW);
                            }
                            "REPEATABLE READ" => {
                                e.token(TokenKind::REPEATABLE_KW);
                                e.space();
                                e.token(TokenKind::READ_KW);
                            }
                            "READ COMMITTED" => {
                                e.token(TokenKind::READ_KW);
                                e.space();
                                e.token(TokenKind::COMMITTED_KW);
                            }
                            "READ UNCOMMITTED" => {
                                e.token(TokenKind::READ_KW);
                                e.space();
                                e.token(TokenKind::UNCOMMITTED_KW);
                            }
                            _ => {
                                e.token(TokenKind::IDENT(level));
                            }
                        }
                    }
                }
            }
        }
        "transaction_read_only" => {
            // arg is boolean: true = READ ONLY, false = READ WRITE
            let is_read_only = if let Some(ref arg) = def.arg {
                if let Some(NodeEnum::Boolean(b)) = &arg.node {
                    b.boolval
                } else if let Some(NodeEnum::Integer(i)) = &arg.node {
                    i.ival != 0
                } else if let Some(NodeEnum::AConst(ac)) = &arg.node {
                    match &ac.val {
                        Some(pgls_query::protobuf::a_const::Val::Ival(i)) => i.ival != 0,
                        Some(pgls_query::protobuf::a_const::Val::Boolval(b)) => b.boolval,
                        _ => true,
                    }
                } else {
                    true
                }
            } else {
                true
            };
            e.token(TokenKind::READ_KW);
            e.space();
            if is_read_only {
                e.token(TokenKind::ONLY_KW);
            } else {
                e.token(TokenKind::WRITE_KW);
            }
        }
        "transaction_deferrable" => {
            // arg is boolean: true = DEFERRABLE, false = NOT DEFERRABLE
            let is_deferrable = if let Some(ref arg) = def.arg {
                if let Some(NodeEnum::Boolean(b)) = &arg.node {
                    b.boolval
                } else if let Some(NodeEnum::Integer(i)) = &arg.node {
                    i.ival != 0
                } else if let Some(NodeEnum::AConst(ac)) = &arg.node {
                    match &ac.val {
                        Some(pgls_query::protobuf::a_const::Val::Ival(i)) => i.ival != 0,
                        Some(pgls_query::protobuf::a_const::Val::Boolval(b)) => b.boolval,
                        _ => true,
                    }
                } else {
                    true
                }
            } else {
                true
            };
            if !is_deferrable {
                e.token(TokenKind::NOT_KW);
                e.space();
            }
            e.token(TokenKind::DEFERRABLE_KW);
        }
        _ => {
            // Unknown transaction option - emit as identifier
            e.token(TokenKind::IDENT(def.defname.to_uppercase()));
            if let Some(ref arg) = def.arg {
                e.space();
                super::emit_node(arg, e);
            }
        }
    }
}
