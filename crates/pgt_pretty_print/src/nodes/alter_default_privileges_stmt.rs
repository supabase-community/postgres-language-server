use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterDefaultPrivilegesStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_default_privileges_stmt(
    e: &mut EventEmitter,
    n: &AlterDefaultPrivilegesStmt,
) {
    e.group_start(GroupKind::AlterDefaultPrivilegesStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DEFAULT_KW);
    e.space();
    e.token(TokenKind::IDENT("PRIVILEGES".to_string()));

    // Optional: FOR ROLE/USER or IN SCHEMA
    if !n.options.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }

    // The actual GRANT/REVOKE statement
    if let Some(ref action) = n.action {
        e.space();
        super::emit_node_enum(&pgt_query::NodeEnum::GrantStmt(action.clone()), e);
    }

    e.group_end();
}
