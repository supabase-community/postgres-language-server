use pgls_query::protobuf::AlterTypeStmt;

use super::prelude::*;

pub(super) fn emit_alter_type_stmt(e: &mut EventEmitter, n: &AlterTypeStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);

    if !n.type_name.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.type_name);
    }

    // ALTER TYPE ... SET (option = value, ...)
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, emit_node);
    }

    e.group_end();
}
