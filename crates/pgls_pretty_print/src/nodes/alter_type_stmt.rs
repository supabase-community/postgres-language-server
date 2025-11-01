use pgls_query::protobuf::AlterTypeStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::{emit_dot_separated_list, emit_space_separated_list},
};

pub(super) fn emit_alter_type_stmt(e: &mut EventEmitter, n: &AlterTypeStmt) {
    e.group_start(GroupKind::AlterTypeStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);

    if !n.type_name.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.type_name);
    }

    if !n.options.is_empty() {
        e.space();
        emit_space_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
