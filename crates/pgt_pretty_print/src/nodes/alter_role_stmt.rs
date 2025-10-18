use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterRoleStmt;

use super::node_list::emit_comma_separated_list;
use super::role_spec::emit_role_spec;

pub(super) fn emit_alter_role_stmt(e: &mut EventEmitter, n: &AlterRoleStmt) {
    e.group_start(GroupKind::AlterRoleStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // action: 1 = ALTER ROLE, -1 = DROP ROLE (but DROP is handled separately)
    e.token(TokenKind::IDENT("ROLE".to_string()));
    e.space();

    if let Some(ref role) = n.role {
        emit_role_spec(e, role);
    }

    // Emit role options
    if !n.options.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
