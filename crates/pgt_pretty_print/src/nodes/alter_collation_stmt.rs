use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterCollationStmt;

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_alter_collation_stmt(e: &mut EventEmitter, n: &AlterCollationStmt) {
    e.group_start(GroupKind::AlterCollationStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("COLLATION".to_string()));
    e.space();

    if !n.collname.is_empty() {
        emit_dot_separated_list(e, &n.collname);
    }

    e.space();
    e.token(TokenKind::IDENT("REFRESH".to_string()));
    e.space();
    e.token(TokenKind::IDENT("VERSION".to_string()));

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
