use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CheckPointStmt;

pub(super) fn emit_checkpoint_stmt(e: &mut EventEmitter, _n: &CheckPointStmt) {
    e.group_start(GroupKind::CheckPointStmt);

    e.token(TokenKind::IDENT("CHECKPOINT".to_string()));
    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
