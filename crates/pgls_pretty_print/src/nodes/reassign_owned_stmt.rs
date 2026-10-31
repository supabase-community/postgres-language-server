use pgls_query::protobuf::ReassignOwnedStmt;

use super::prelude::*;

pub(super) fn emit_reassign_owned_stmt(e: &mut EventEmitter, n: &ReassignOwnedStmt) {
    e.group_start();

    e.token(TokenKind::REASSIGN_KW);
    e.space();
    e.token(TokenKind::OWNED_KW);
    e.space();
    e.token(TokenKind::BY_KW);
    e.space();

    // Emit role list
    emit_comma_separated_list(e, &n.roles, emit_node);

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::TO_KW);
    e.space();

    if let Some(ref newrole) = n.newrole {
        super::emit_role_spec(e, newrole);
    }

    e.group_end();
}
