use pgls_query::protobuf::{CreateRoleStmt, RoleStmtType};

use super::string::emit_identifier_maybe_quoted;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_create_role_stmt(e: &mut EventEmitter, n: &CreateRoleStmt) {
    e.group_start(GroupKind::CreateRoleStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    match n.stmt_type() {
        RoleStmtType::RolestmtRole => e.token(TokenKind::ROLE_KW),
        RoleStmtType::RolestmtUser => e.token(TokenKind::USER_KW),
        RoleStmtType::RolestmtGroup => e.token(TokenKind::GROUP_KW),
        RoleStmtType::Undefined => e.token(TokenKind::ROLE_KW),
    }

    if !n.role.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.role);
    }

    // Process role options with special formatting
    if !n.options.is_empty() {
        e.indent_start();
        for option in &n.options {
            if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = &option.node {
                e.line(LineType::SoftOrSpace);
                format_role_option(e, def_elem);
            }
        }
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn format_role_option(e: &mut EventEmitter, d: &pgls_query::protobuf::DefElem) {
    let defname_lower = d.defname.to_lowercase();

    match defname_lower.as_str() {
        "canlogin" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("login".to_string()));
                } else {
                    e.token(TokenKind::IDENT("nologin".to_string()));
                }
                return;
            }
        }
        "inherit" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::INHERIT_KW);
                } else {
                    e.token(TokenKind::IDENT("noinherit".to_string()));
                }
                return;
            }
        }
        "createrole" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("createrole".to_string()));
                } else {
                    e.token(TokenKind::IDENT("nocreaterole".to_string()));
                }
                return;
            }
        }
        "createdb" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("createdb".to_string()));
                } else {
                    e.token(TokenKind::IDENT("nocreatedb".to_string()));
                }
                return;
            }
        }
        "isreplication" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("replication".to_string()));
                } else {
                    e.token(TokenKind::IDENT("noreplication".to_string()));
                }
                return;
            }
        }
        "issuperuser" | "superuser" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("superuser".to_string()));
                } else {
                    e.token(TokenKind::IDENT("nosuperuser".to_string()));
                }
                return;
            }
        }
        "bypassrls" => {
            if let Some(ref arg) = d.arg
                && let Some(pgls_query::NodeEnum::Boolean(b)) = &arg.node
            {
                if b.boolval {
                    e.token(TokenKind::IDENT("bypassrls".to_string()));
                } else {
                    e.token(TokenKind::IDENT("nobypassrls".to_string()));
                }
                return;
            }
        }
        "connectionlimit" => {
            if let Some(ref arg) = d.arg {
                e.token(TokenKind::CONNECTION_KW);
                e.space();
                e.token(TokenKind::LIMIT_KW);
                e.space();
                super::emit_node(arg, e);
                return;
            }
        }
        "validuntil" => {
            if let Some(ref arg) = d.arg {
                e.token(TokenKind::VALID_KW);
                e.space();
                e.token(TokenKind::UNTIL_KW);
                e.space();
                super::emit_node(arg, e);
                return;
            }
        }
        "addroleto" => {
            if let Some(ref arg) = d.arg {
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::ROLE_KW);
                e.space();
                super::emit_node(arg, e);
                return;
            }
        }
        "rolemembers" => {
            if let Some(ref arg) = d.arg {
                e.token(TokenKind::ROLE_KW);
                e.space();
                super::emit_node(arg, e);
                return;
            }
        }
        "adminmembers" => {
            if let Some(ref arg) = d.arg {
                e.token(TokenKind::ADMIN_KW);
                e.space();
                super::emit_node(arg, e);
                return;
            }
        }
        "password" => {
            e.token(TokenKind::PASSWORD_KW);
            e.space();
            if let Some(ref arg) = d.arg {
                // Password must be a string literal with single quotes
                if let Some(pgls_query::NodeEnum::String(s)) = &arg.node {
                    super::emit_string_literal(e, s);
                } else {
                    super::emit_node(arg, e);
                }
            } else {
                e.token(TokenKind::NULL_KW);
            }
            return;
        }
        _ => {}
    }

    // Default formatting for other options
    let defname_upper = d.defname.to_uppercase();
    e.token(TokenKind::IDENT(defname_upper));
    if let Some(ref arg) = d.arg {
        e.space();
        super::emit_node(arg, e);
    }
}
