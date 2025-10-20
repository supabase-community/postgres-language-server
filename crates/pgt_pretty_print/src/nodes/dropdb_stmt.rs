use pgt_query::protobuf::DropdbStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_dropdb_stmt(e: &mut EventEmitter, n: &DropdbStmt) {
    e.group_start(GroupKind::DropdbStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::DATABASE_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.dbname.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.dbname.clone()));
    }

    // Note: options field exists but not commonly used - skipping for now

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
