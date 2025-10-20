use pgt_query::protobuf::DeallocateStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_deallocate_stmt(e: &mut EventEmitter, n: &DeallocateStmt) {
    e.group_start(GroupKind::DeallocateStmt);

    e.token(TokenKind::DEALLOCATE_KW);
    e.space();

    if n.name.is_empty() || n.name == "ALL" {
        e.token(TokenKind::ALL_KW);
    } else {
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
