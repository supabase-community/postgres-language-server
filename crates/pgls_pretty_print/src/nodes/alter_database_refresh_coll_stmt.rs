use pgls_query::protobuf::AlterDatabaseRefreshCollStmt;

use super::prelude::*;

pub(super) fn emit_alter_database_refresh_coll_stmt(
    e: &mut EventEmitter,
    n: &AlterDatabaseRefreshCollStmt,
) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);
    e.space();

    if !n.dbname.is_empty() {
        emit_identifier_maybe_quoted(e, &n.dbname);
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::REFRESH_KW);
    e.space();
    e.token(TokenKind::COLLATION_KW);
    e.space();
    e.token(TokenKind::VERSION_KW);

    e.group_end();
}
