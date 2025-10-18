use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::AlterDatabaseRefreshCollStmt;

pub(super) fn emit_alter_database_refresh_coll_stmt(
    e: &mut EventEmitter,
    n: &AlterDatabaseRefreshCollStmt,
) {
    e.group_start(GroupKind::AlterDatabaseRefreshCollStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("DATABASE".to_string()));
    e.space();

    if !n.dbname.is_empty() {
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    e.space();
    e.token(TokenKind::IDENT("REFRESH".to_string()));
    e.space();
    e.token(TokenKind::IDENT("COLLATION".to_string()));
    e.space();
    e.token(TokenKind::IDENT("VERSION".to_string()));

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
