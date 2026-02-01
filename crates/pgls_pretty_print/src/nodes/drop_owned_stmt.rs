use pgls_query::protobuf::DropOwnedStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_drop_owned_stmt(e: &mut EventEmitter, n: &DropOwnedStmt) {
    e.group_start(GroupKind::DropOwnedStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::OWNED_KW);
    e.space();
    e.token(TokenKind::BY_KW);

    // Role list
    if !n.roles.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.roles, super::emit_node);
    }

    // CASCADE or RESTRICT
    // behavior: 0=Undefined, 1=DropRestrict, 2=DropCascade
    // RESTRICT is the default, so only emit CASCADE explicitly
    if n.behavior == 2 {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::CASCADE_KW);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
