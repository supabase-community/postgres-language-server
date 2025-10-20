use pgt_query::protobuf::ExecuteStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::emit_node;

pub(super) fn emit_execute_stmt(e: &mut EventEmitter, n: &ExecuteStmt) {
    e.group_start(GroupKind::ExecuteStmt);

    e.token(TokenKind::EXECUTE_KW);
    e.space();
    e.token(TokenKind::IDENT(n.name.clone()));

    if !n.params.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.params, emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
