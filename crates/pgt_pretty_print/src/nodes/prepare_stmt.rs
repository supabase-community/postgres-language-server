use pgt_query::protobuf::PrepareStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

pub(super) fn emit_prepare_stmt(e: &mut EventEmitter, n: &PrepareStmt) {
    e.group_start(GroupKind::PrepareStmt);

    e.token(TokenKind::PREPARE_KW);

    // Statement name
    if !n.name.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    // Argument types
    if !n.argtypes.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.argtypes, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    // AS query
    if let Some(ref query) = n.query {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_node(query, e);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
