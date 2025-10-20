use pgt_query::protobuf::DropTableSpaceStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_drop_table_space_stmt(e: &mut EventEmitter, n: &DropTableSpaceStmt) {
    e.group_start(GroupKind::DropTableSpaceStmt);

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::TABLESPACE_KW);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    if !n.tablespacename.is_empty() {
        e.space();
        e.token(TokenKind::IDENT(n.tablespacename.clone()));
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
