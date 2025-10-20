use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::{
    NodeEnum,
    protobuf::{Node, VariableSetStmt},
};

/// Emit a SET statement argument
/// Special handling: AConst with string values should be emitted as unquoted identifiers
fn emit_set_arg(node: &Node, e: &mut EventEmitter) {
    if let Some(NodeEnum::AConst(a_const)) = &node.node {
        if let Some(pgt_query::protobuf::a_const::Val::Sval(s)) = &a_const.val {
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
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

pub(super) fn emit_variable_set_stmt(e: &mut EventEmitter, n: &VariableSetStmt) {
    e.group_start(GroupKind::VariableSetStmt);

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
        e.token(TokenKind::IDENT(n.name.clone()));
        e.token(TokenKind::SEMICOLON);
        e.group_end();
        return;
    } else if n.kind == 6 {
        // VAR_RESET_ALL - emit RESET ALL
        e.token(TokenKind::RESET_KW);
        e.space();
        e.token(TokenKind::ALL_KW);
        e.token(TokenKind::SEMICOLON);
        e.group_end();
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
            "client_encoding" => {
                e.token(TokenKind::NAMES_KW);
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
                // Generic variable name
                e.token(TokenKind::IDENT(n.name.clone()));
            }
        }

        // Emit value assignment
        if !n.args.is_empty() {
            // Determine whether to use = or TO or nothing
            // SESSION AUTHORIZATION uses no connector (just space)
            // Most special variables use TO
            // Generic variables use =
            let uses_to = matches!(
                n.name.to_lowercase().as_str(),
                "timezone"
                    | "catalog"
                    | "client_encoding"
                    | "role"
                    | "transaction_isolation"
                    | "search_path"
            );

            let no_connector = n.name.to_lowercase() == "session_authorization";

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
                    | "client_encoding"
                    | "role"
                    | "transaction_isolation"
                    | "search_path"
            );

            e.token(TokenKind::IDENT(n.name.clone()));
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
        e.token(TokenKind::IDENT(n.name.clone()));
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::CURRENT_KW);
    } else {
        // VAR_SET_MULTI, VAR_RESET, VAR_RESET_ALL or other
        // TODO: Handle these variants properly
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
