use pgt_query::protobuf::ReassignOwnedStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::emit_node;

pub(super) fn emit_reassign_owned_stmt(e: &mut EventEmitter, n: &ReassignOwnedStmt) {
    e.group_start(GroupKind::ReassignOwnedStmt);

    e.token(TokenKind::REASSIGN_KW);
    e.space();
    e.token(TokenKind::OWNED_KW);
    e.space();
    e.token(TokenKind::BY_KW);
    e.space();

    // Emit role list
    super::node_list::emit_comma_separated_list(e, &n.roles, emit_node);

    e.space();
    e.token(TokenKind::TO_KW);
    e.space();

    if let Some(ref newrole) = n.newrole {
        super::emit_role_spec(e, newrole);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
