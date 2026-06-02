use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterDatabaseSetStmt;

pub(super) fn emit_alter_database_set_stmt(e: &mut EventEmitter, n: &AlterDatabaseSetStmt) {
    e.group_start(GroupKind::AlterDatabaseSetStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);
    e.space();

    if !n.dbname.is_empty() {
        super::emit_identifier_maybe_quoted(e, &n.dbname);
    }

    if let Some(ref setstmt) = n.setstmt {
        e.line(LineType::SoftOrSpace);
        super::emit_variable_set_stmt(e, setstmt);
    }

    e.group_end();
}
