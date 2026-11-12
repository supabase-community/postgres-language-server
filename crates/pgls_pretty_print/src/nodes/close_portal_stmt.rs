use pgls_query::protobuf::ClosePortalStmt;

use super::prelude::*;

pub(super) fn emit_close_portal_stmt(e: &mut EventEmitter, n: &ClosePortalStmt) {
    e.group_start();

    e.token(TokenKind::CLOSE_KW);
    e.space();

    if n.portalname.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else {
        emit_identifier_maybe_quoted(e, &n.portalname);
    }

    e.group_end();
}
