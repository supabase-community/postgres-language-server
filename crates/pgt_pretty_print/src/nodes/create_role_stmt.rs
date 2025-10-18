use pgt_query::protobuf::{CreateRoleStmt, RoleStmtType};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

pub(super) fn emit_create_role_stmt(e: &mut EventEmitter, n: &CreateRoleStmt) {
    e.group_start(GroupKind::CreateRoleStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    let stmt_type = RoleStmtType::try_from(n.stmt_type).unwrap_or(RoleStmtType::Undefined);
    match stmt_type {
        RoleStmtType::RolestmtRole => e.token(TokenKind::ROLE_KW),
        RoleStmtType::RolestmtUser => e.token(TokenKind::USER_KW),
        RoleStmtType::RolestmtGroup => e.token(TokenKind::GROUP_KW),
        RoleStmtType::Undefined => e.token(TokenKind::ROLE_KW),
    }

    if !n.role.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.role.clone()));
    }

    // Process role options with special formatting
    if !n.options.is_empty() {
        e.indent_start();
        for option in &n.options {
            if let Some(ref node) = option.node {
                if let pgt_query::NodeEnum::DefElem(def_elem) = node {
                    e.line(LineType::SoftOrSpace);
                    format_role_option(e, def_elem);
                }
            }
        }
        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn format_role_option(e: &mut EventEmitter, d: &pgt_query::protobuf::DefElem) {
    let defname_lower = d.defname.to_lowercase();

    match defname_lower.as_str() {
        "canlogin" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("LOGIN".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOLOGIN".to_string()));
                    }
                    return;
                }
            }
        }
        "inherit" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::INHERIT_KW);
                    } else {
                        e.token(TokenKind::IDENT("NOINHERIT".to_string()));
                    }
                    return;
                }
            }
        }
        "createrole" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("CREATEROLE".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOCREATEROLE".to_string()));
                    }
                    return;
                }
            }
        }
        "createdb" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("CREATEDB".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOCREATEDB".to_string()));
                    }
                    return;
                }
            }
        }
        "isreplication" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("REPLICATION".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOREPLICATION".to_string()));
                    }
                    return;
                }
            }
        }
        "issuperuser" | "superuser" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("SUPERUSER".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOSUPERUSER".to_string()));
                    }
                    return;
                }
            }
        }
        "bypassrls" => {
            if let Some(ref arg) = d.arg {
                if let Some(pgt_query::NodeEnum::Boolean(b)) = &arg.node {
                    if b.boolval {
                        e.token(TokenKind::IDENT("BYPASSRLS".to_string()));
                    } else {
                        e.token(TokenKind::IDENT("NOBYPASSRLS".to_string()));
                    }
                    return;
                }
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
                if let Some(pgt_query::NodeEnum::String(s)) = &arg.node {
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
