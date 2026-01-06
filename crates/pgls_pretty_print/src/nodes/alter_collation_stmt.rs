use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterCollationStmt;

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_alter_collation_stmt(e: &mut EventEmitter, n: &AlterCollationStmt) {
    e.group_start(GroupKind::AlterCollationStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::COLLATION_KW);
    e.space();

    if !n.collname.is_empty() {
        emit_dot_separated_list(e, &n.collname);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::REFRESH_KW);
    e.space();
    e.token(TokenKind::VERSION_KW);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
