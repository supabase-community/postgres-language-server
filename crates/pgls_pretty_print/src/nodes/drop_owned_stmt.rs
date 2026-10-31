use pgls_query::protobuf::DropOwnedStmt;

use super::prelude::*;

pub(super) fn emit_drop_owned_stmt(e: &mut EventEmitter, n: &DropOwnedStmt) {
    e.group_start();

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::OWNED_KW);
    e.space();
    e.token(TokenKind::BY_KW);

    // Role list
    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.roles, emit_node);
    }

    emit_drop_behavior(e, n.behavior(), Separator::SoftOrSpace, Restrict::Omitted);

    e.group_end();
}
