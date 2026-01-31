use pgls_query::protobuf::ExecuteStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::emit_node;

pub(super) fn emit_execute_stmt(e: &mut EventEmitter, n: &ExecuteStmt) {
    emit_execute_stmt_impl(e, n, true);
}

pub(super) fn emit_execute_stmt_no_semicolon(e: &mut EventEmitter, n: &ExecuteStmt) {
    emit_execute_stmt_impl(e, n, false);
}

fn emit_execute_stmt_impl(e: &mut EventEmitter, n: &ExecuteStmt, with_semicolon: bool) {
    e.group_start(GroupKind::ExecuteStmt);

    e.token(TokenKind::EXECUTE_KW);
    e.space();
    e.token(TokenKind::IDENT(n.name.clone()));

    if !n.params.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::L_PAREN);
        super::node_list::emit_comma_separated_list(e, &n.params, emit_node);
        e.token(TokenKind::R_PAREN);
    }

    if with_semicolon {
        e.token(TokenKind::SEMICOLON);
    }

    e.group_end();
}
