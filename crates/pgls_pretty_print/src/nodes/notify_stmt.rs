use pgls_query::protobuf::NotifyStmt;

use super::prelude::*;

pub(super) fn emit_notify_stmt(e: &mut EventEmitter, n: &NotifyStmt) {
    e.group_start(GroupKind::NotifyStmt);

    e.token(TokenKind::NOTIFY_KW);

    if !n.conditionname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.conditionname);
    }

    // Optional payload
    if !n.payload.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::COMMA);
        e.space();
        emit_single_quoted_str(e, &n.payload);
    }

    e.group_end();
}
