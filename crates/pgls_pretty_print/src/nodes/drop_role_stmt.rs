use pgls_query::protobuf::DropRoleStmt;

use super::prelude::*;

pub(super) fn emit_drop_role_stmt(e: &mut EventEmitter, n: &DropRoleStmt) {
    e.group_start();

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::ROLE_KW);

    if n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.roles, emit_node);
    }

    e.group_end();
}
