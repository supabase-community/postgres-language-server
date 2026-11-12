use pgls_query::protobuf::GrantRoleStmt;

use super::prelude::*;

pub(super) fn emit_grant_role_stmt(e: &mut EventEmitter, n: &GrantRoleStmt) {
    e.group_start();

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
                // Emit the option name
                emit_keyword(e, &def.defname);
                e.space();
                e.token(TokenKind::OPTION_KW);
                e.space();
                e.token(TokenKind::FOR_KW);
            }
        }
    }

    // Role list
    if !n.granted_roles.is_empty() {
        e.space();
        emit_grant_list(e, &n.granted_roles, emit_node);
    }

    // The clauses after the granted roles continue the statement, so a broken one indents them.
    e.indent_start();

    // TO or FROM
    if let Some(grantee) = n.grantee_roles.first() {
        emit_comments_before_keyword(e, if n.is_grant { "to" } else { "from" }, grantee);
    }
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
        emit_grant_list(e, &n.grantee_roles, emit_node);
    }

    // WITH options (WITH INHERIT TRUE, SET FALSE, etc.) - only for GRANT
    // Syntax: WITH option1 TRUE, option2 FALSE
    if !n.opt.is_empty() && n.is_grant {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_grant_list(e, &n.opt, |e, opt| {
            if let Some(NodeEnum::DefElem(def)) = opt.node.as_ref() {
                emit_grant_role_option(e, def);
            }
        });
    }

    // GRANTED BY
    if let Some(ref grantor) = n.grantor {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::GRANTED_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::emit_role_spec(e, grantor);
    }

    // CASCADE or RESTRICT (for REVOKE)
    if !n.is_grant {
        emit_drop_behavior(e, n.behavior(), Separator::SoftOrSpace, Restrict::Written);
    }

    e.indent_end();

    e.group_end();
}
