use pgt_query::protobuf::ReindexStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_reindex_stmt(e: &mut EventEmitter, n: &ReindexStmt) {
    e.group_start(GroupKind::ReindexStmt);

    e.token(TokenKind::REINDEX_KW);
    e.space();

    // ReindexObjectType enum:
    // 0: REINDEX_OBJECT_INDEX
    // 1: REINDEX_OBJECT_TABLE
    // 2: REINDEX_OBJECT_SCHEMA
    // 3: REINDEX_OBJECT_SYSTEM
    // 4: REINDEX_OBJECT_DATABASE
    match n.kind {
        0 => e.token(TokenKind::INDEX_KW),
        1 => e.token(TokenKind::TABLE_KW),
        2 => e.token(TokenKind::SCHEMA_KW),
        3 => e.token(TokenKind::SYSTEM_KW),
        4 => e.token(TokenKind::DATABASE_KW),
        _ => e.token(TokenKind::TABLE_KW), // default
    }

    e.space();

    // Either relation or name is used
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    } else if !n.name.is_empty() {
        e.token(TokenKind::IDENT(n.name.clone()));
    }

    // TODO: Handle params (options like CONCURRENTLY, VERBOSE)

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
