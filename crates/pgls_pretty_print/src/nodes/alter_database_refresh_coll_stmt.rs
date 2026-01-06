use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};
use pgls_query::protobuf::AlterDatabaseRefreshCollStmt;

pub(super) fn emit_alter_database_refresh_coll_stmt(
    e: &mut EventEmitter,
    n: &AlterDatabaseRefreshCollStmt,
) {
    e.group_start(GroupKind::AlterDatabaseRefreshCollStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);
    e.space();

    if !n.dbname.is_empty() {
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::REFRESH_KW);
    e.space();
    e.token(TokenKind::COLLATION_KW);
    e.space();
    e.token(TokenKind::VERSION_KW);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
