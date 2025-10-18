use pgt_query::protobuf::GrantRoleStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_grant_role_stmt(e: &mut EventEmitter, n: &GrantRoleStmt) {
    e.group_start(GroupKind::GrantRoleStmt);

    // GRANT or REVOKE
    if n.is_grant {
        e.token(TokenKind::GRANT_KW);
    } else {
        e.token(TokenKind::REVOKE_KW);
    }

    // Role list
    if !n.granted_roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.granted_roles, super::emit_node);
    }

    // TO or FROM
    if n.is_grant {
        e.space();
        e.token(TokenKind::TO_KW);
    } else {
        e.space();
        e.token(TokenKind::FROM_KW);
    }

    // Grantee list
    if !n.grantee_roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.grantee_roles, super::emit_node);
    }

    // WITH options (WITH ADMIN OPTION, etc.)
    if !n.opt.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_comma_separated_list(e, &n.opt, super::emit_node);
    }

    // GRANTED BY
    if let Some(ref grantor) = n.grantor {
        e.space();
        e.token(TokenKind::IDENT("GRANTED".to_string()));
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::emit_role_spec(e, grantor);
    }

    // CASCADE or RESTRICT (for REVOKE)
    if !n.is_grant && n.behavior != 0 {
        e.space();
        match n.behavior {
            1 => e.token(TokenKind::CASCADE_KW),
            _ => e.token(TokenKind::RESTRICT_KW),
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
