use pgls_query::protobuf::AlterDatabaseStmt;

use super::prelude::*;

pub(super) fn emit_alter_database_stmt(e: &mut EventEmitter, n: &AlterDatabaseStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);
    e.space();

    if !n.dbname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.dbname);
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        // The options are space-separated, as in CREATE DATABASE.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L11456
        emit_space_separated_list(e, &n.options, emit_node);
    }

    e.group_end();
}
