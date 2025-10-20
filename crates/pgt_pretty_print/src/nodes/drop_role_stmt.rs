use pgt_query::protobuf::DropRoleStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_drop_role_stmt(e: &mut EventEmitter, n: &DropRoleStmt) {
    e.group_start(GroupKind::DropRoleStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::ROLE_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.roles, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
