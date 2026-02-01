use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{emit_comma_separated_list, emit_dot_separated_list},
};
use pgls_query::protobuf::CreateRangeStmt;

pub(super) fn emit_create_range_stmt(e: &mut EventEmitter, n: &CreateRangeStmt) {
    e.group_start(GroupKind::CreateRangeStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);
    e.space();

    emit_dot_separated_list(e, &n.type_name);

    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    e.token(TokenKind::IDENT("range".to_string()));

    if !n.params.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        emit_comma_separated_list(e, &n.params, super::emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
