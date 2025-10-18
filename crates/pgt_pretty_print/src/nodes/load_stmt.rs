use pgt_query::protobuf::LoadStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::string::emit_single_quoted_str;

pub(super) fn emit_load_stmt(e: &mut EventEmitter, n: &LoadStmt) {
    e.group_start(GroupKind::LoadStmt);

    e.token(TokenKind::LOAD_KW);

    if !n.filename.is_empty() {
        e.space();
        emit_single_quoted_str(e, &n.filename);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
