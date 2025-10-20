use pgt_query::protobuf::NotifyStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::string::emit_single_quoted_str;

pub(super) fn emit_notify_stmt(e: &mut EventEmitter, n: &NotifyStmt) {
    e.group_start(GroupKind::NotifyStmt);

    e.token(TokenKind::NOTIFY_KW);

    if !n.conditionname.is_empty() {
        e.space();
        super::emit_identifier(e, &n.conditionname);
    }

    // Optional payload
    if !n.payload.is_empty() {
        e.space();
        e.token(TokenKind::COMMA);
        e.space();
        emit_single_quoted_str(e, &n.payload);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
