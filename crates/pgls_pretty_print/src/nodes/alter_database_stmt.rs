use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterDatabaseStmt;

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_alter_database_stmt(e: &mut EventEmitter, n: &AlterDatabaseStmt) {
    e.group_start(GroupKind::AlterDatabaseStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);
    e.space();

    if !n.dbname.is_empty() {
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
