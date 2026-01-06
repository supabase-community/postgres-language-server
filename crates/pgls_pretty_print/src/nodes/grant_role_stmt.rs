use pgls_query::{NodeEnum, protobuf::GrantRoleStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

use super::def_elem::emit_grant_role_option;

pub(super) fn emit_grant_role_stmt(e: &mut EventEmitter, n: &GrantRoleStmt) {
    e.group_start(GroupKind::GrantRoleStmt);

    // GRANT or REVOKE
    if n.is_grant {
        e.token(TokenKind::GRANT_KW);
    } else {
        e.token(TokenKind::REVOKE_KW);

        // For REVOKE, options like inherit=false become "INHERIT OPTION FOR"
        // REVOKE [ADMIN | INHERIT | SET] OPTION FOR role_name FROM ...
        for opt in &n.opt {
            if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
                e.space();
                // Emit the option name as uppercase
                let opt_name = match def.defname.as_str() {
                    "admin" => "ADMIN",
                    "inherit" => "INHERIT",
                    "set" => "SET",
                    _ => &def.defname.to_uppercase(),
                };
                e.token(TokenKind::IDENT(opt_name.to_string()));
                e.space();
                e.token(TokenKind::IDENT("OPTION".to_string()));
                e.space();
                e.token(TokenKind::FOR_KW);
            }
        }
    }

    // Role list
    if !n.granted_roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.granted_roles, super::emit_node);
    }

    // TO or FROM
    if n.is_grant {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::TO_KW);
    } else {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
    }

    // Grantee list
    if !n.grantee_roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.grantee_roles, super::emit_node);
    }

    // WITH options (WITH INHERIT TRUE, SET FALSE, etc.) - only for GRANT
    // Syntax: WITH option1 TRUE, option2 FALSE
    if !n.opt.is_empty() && n.is_grant {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        let mut first = true;
        for opt in &n.opt {
            if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
                if first {
                    e.space();
                    first = false;
                } else {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                emit_grant_role_option(e, def);
            }
        }
    }

    // GRANTED BY
    if let Some(ref grantor) = n.grantor {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("GRANTED".to_string()));
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::emit_role_spec(e, grantor);
    }

    // CASCADE or RESTRICT (for REVOKE)
    // DropBehavior: 0=Undefined (default, omit), 1=DropRestrict, 2=DropCascade
    if !n.is_grant {
        match n.behavior {
            1 => {
                // DropRestrict - RESTRICT is the default, but emit for explicitness
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::RESTRICT_KW);
            }
            2 => {
                // DropCascade
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::CASCADE_KW);
            }
            _ => {
                // Undefined - don't emit anything (defaults to RESTRICT in PostgreSQL)
            }
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
