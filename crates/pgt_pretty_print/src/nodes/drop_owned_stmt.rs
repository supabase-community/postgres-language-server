use pgt_query::protobuf::DropOwnedStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_drop_owned_stmt(e: &mut EventEmitter, n: &DropOwnedStmt) {
    e.group_start(GroupKind::DropOwnedStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::IDENT("OWNED".to_string()));
    e.space();
    e.token(TokenKind::BY_KW);

    // Role list
    if !n.roles.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.roles, super::emit_node);
    }

    // CASCADE or RESTRICT
    if n.behavior != 0 {
        e.space();
        match n.behavior {
            1 => e.token(TokenKind::CASCADE_KW),
            _ => e.token(TokenKind::RESTRICT_KW),
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
