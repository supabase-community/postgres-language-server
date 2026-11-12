use pgls_query::protobuf::DropSubscriptionStmt;

use super::prelude::*;

pub(super) fn emit_drop_subscription_stmt(e: &mut EventEmitter, n: &DropSubscriptionStmt) {
    e.group_start();

    e.token(TokenKind::DROP_KW);
    e.space();
    e.token(TokenKind::SUBSCRIPTION_KW);

    if n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    if !n.subname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.subname);
    }

    emit_drop_behavior(e, n.behavior(), Separator::Space, Restrict::Written);

    e.group_end();
}
