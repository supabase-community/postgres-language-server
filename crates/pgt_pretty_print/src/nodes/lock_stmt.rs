use pgt_query::protobuf::LockStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::emit_node;

pub(super) fn emit_lock_stmt(e: &mut EventEmitter, n: &LockStmt) {
    e.group_start(GroupKind::LockStmt);

    e.token(TokenKind::LOCK_KW);
    e.space();
    e.token(TokenKind::TABLE_KW);
    e.space();

    // Emit table list
    super::node_list::emit_comma_separated_list(e, &n.relations, emit_node);

    // Lock mode - mapping from AccessExclusiveLock enum (1-8)
    // 1: AccessShareLock -> ACCESS SHARE
    // 2: RowShareLock -> ROW SHARE
    // 3: RowExclusiveLock -> ROW EXCLUSIVE
    // 4: ShareUpdateExclusiveLock -> SHARE UPDATE EXCLUSIVE
    // 5: ShareLock -> SHARE
    // 6: ShareRowExclusiveLock -> SHARE ROW EXCLUSIVE
    // 7: ExclusiveLock -> EXCLUSIVE
    // 8: AccessExclusiveLock -> ACCESS EXCLUSIVE
    if n.mode > 0 {
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        let mode_str = match n.mode {
            1 => "ACCESS SHARE",
            2 => "ROW SHARE",
            3 => "ROW EXCLUSIVE",
            4 => "SHARE UPDATE EXCLUSIVE",
            5 => "SHARE",
            6 => "SHARE ROW EXCLUSIVE",
            7 => "EXCLUSIVE",
            8 => "ACCESS EXCLUSIVE",
            _ => "ACCESS EXCLUSIVE", // default
        };
        e.token(TokenKind::IDENT(mode_str.to_string()));
        e.space();
        e.token(TokenKind::MODE_KW);
    }

    if n.nowait {
        e.space();
        e.token(TokenKind::IDENT("NOWAIT".to_string()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
