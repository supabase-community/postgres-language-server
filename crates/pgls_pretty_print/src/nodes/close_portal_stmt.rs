use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::ClosePortalStmt;

pub(super) fn emit_close_portal_stmt(e: &mut EventEmitter, n: &ClosePortalStmt) {
    e.group_start(GroupKind::ClosePortalStmt);

    e.token(TokenKind::CLOSE_KW);
    e.space();

    if n.portalname.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else {
        super::emit_identifier_maybe_quoted(e, &n.portalname);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
