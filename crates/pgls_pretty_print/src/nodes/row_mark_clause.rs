use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{LockClauseStrength, LockWaitPolicy, RowMarkClause};
use std::convert::TryFrom;

pub(super) fn emit_row_mark_clause(e: &mut EventEmitter, n: &RowMarkClause) {
    e.group_start(GroupKind::RowMarkClause);

    e.token(TokenKind::FOR_KW);
    e.space();
    emit_strength(e, n.strength);

    if n.rti > 0 {
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();
        super::emit_identifier(e, &format!("rte#{}", n.rti));
    }

    match LockWaitPolicy::try_from(n.wait_policy).unwrap_or(LockWaitPolicy::Undefined) {
        LockWaitPolicy::LockWaitSkip => {
            e.space();
            e.token(TokenKind::IDENT("SKIP".to_string()));
            e.space();
            e.token(TokenKind::IDENT("LOCKED".to_string()));
        }
        LockWaitPolicy::LockWaitError => {
            e.space();
            e.token(TokenKind::IDENT("NOWAIT".to_string()));
        }
        LockWaitPolicy::LockWaitBlock | LockWaitPolicy::Undefined => {}
    }

    if n.pushed_down {
        e.space();
        super::emit_identifier(e, "pushed_down");
    }

    e.group_end();
}

fn emit_strength(e: &mut EventEmitter, raw: i32) {
    match LockClauseStrength::try_from(raw).unwrap_or(LockClauseStrength::Undefined) {
        LockClauseStrength::LcsForupdate => e.token(TokenKind::UPDATE_KW),
        LockClauseStrength::LcsFornokeyupdate => {
            e.token(TokenKind::IDENT("NO".to_string()));
            e.space();
            e.token(TokenKind::KEY_KW);
            e.space();
            e.token(TokenKind::UPDATE_KW);
        }
        LockClauseStrength::LcsForshare => e.token(TokenKind::SHARE_KW),
        LockClauseStrength::LcsForkeyshare => {
            e.token(TokenKind::KEY_KW);
            e.space();
            e.token(TokenKind::SHARE_KW);
        }
        LockClauseStrength::LcsNone | LockClauseStrength::Undefined => {
            super::emit_identifier(e, "lock");
        }
    }
}
