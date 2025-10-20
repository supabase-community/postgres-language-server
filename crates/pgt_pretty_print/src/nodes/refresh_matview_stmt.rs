use pgt_query::protobuf::RefreshMatViewStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_refresh_matview_stmt(e: &mut EventEmitter, n: &RefreshMatViewStmt) {
    e.group_start(GroupKind::RefreshMatViewStmt);

    e.token(TokenKind::REFRESH_KW);
    e.space();
    e.token(TokenKind::MATERIALIZED_KW);
    e.space();
    e.token(TokenKind::VIEW_KW);

    if n.concurrent {
        e.space();
        e.token(TokenKind::CONCURRENTLY_KW);
    }

    e.space();
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    if n.skip_data {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::NO_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
