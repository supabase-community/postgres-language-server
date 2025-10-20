use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterDatabaseSetStmt;

pub(super) fn emit_alter_database_set_stmt(e: &mut EventEmitter, n: &AlterDatabaseSetStmt) {
    e.group_start(GroupKind::AlterDatabaseSetStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("DATABASE".to_string()));
    e.space();

    if !n.dbname.is_empty() {
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    if let Some(ref setstmt) = n.setstmt {
        e.space();
        super::emit_variable_set_stmt(e, setstmt);
    }

    e.group_end();
}
