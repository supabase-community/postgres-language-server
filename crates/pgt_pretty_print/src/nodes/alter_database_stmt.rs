use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterDatabaseStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_database_stmt(e: &mut EventEmitter, n: &AlterDatabaseStmt) {
    e.group_start(GroupKind::AlterDatabaseStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("DATABASE".to_string()));
    e.space();

    if !n.dbname.is_empty() {
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    if !n.options.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
