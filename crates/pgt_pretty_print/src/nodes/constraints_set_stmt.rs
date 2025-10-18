use super::node_list::emit_comma_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::ConstraintsSetStmt;

pub(super) fn emit_constraints_set_stmt(e: &mut EventEmitter, n: &ConstraintsSetStmt) {
    e.group_start(GroupKind::ConstraintsSetStmt);

    e.token(TokenKind::SET_KW);
    e.space();
    e.token(TokenKind::IDENT("CONSTRAINTS".to_string()));
    e.space();

    if n.constraints.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else {
        emit_comma_separated_list(e, &n.constraints, super::emit_node);
    }

    e.space();
    if n.deferred {
        e.token(TokenKind::IDENT("DEFERRED".to_string()));
    } else {
        e.token(TokenKind::IDENT("IMMEDIATE".to_string()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
