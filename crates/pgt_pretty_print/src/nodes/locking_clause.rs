use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{LockClauseStrength, LockWaitPolicy, LockingClause};

use super::{emit_node, node_list::emit_comma_separated_list, string::emit_keyword};

pub(super) fn emit_locking_clause(e: &mut EventEmitter, n: &LockingClause) {
    e.group_start(GroupKind::LockingClause);

    e.token(TokenKind::FOR_KW);
    e.space();

    match n.strength() {
        LockClauseStrength::LcsFornokeyupdate => {
            emit_keyword(e, "NO");
            e.space();
            emit_keyword(e, "KEY");
            e.space();
            emit_keyword(e, "UPDATE");
        }
        LockClauseStrength::LcsForupdate
        | LockClauseStrength::LcsNone
        | LockClauseStrength::Undefined => {
            emit_keyword(e, "UPDATE");
        }
        LockClauseStrength::LcsForshare => {
            emit_keyword(e, "SHARE");
        }
        LockClauseStrength::LcsForkeyshare => {
            emit_keyword(e, "KEY");
            e.space();
            emit_keyword(e, "SHARE");
        }
    }

    if !n.locked_rels.is_empty() {
        e.space();
        e.token(TokenKind::OF_KW);
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        emit_comma_separated_list(e, &n.locked_rels, emit_node);
        e.indent_end();
    }

    match n.wait_policy() {
        LockWaitPolicy::LockWaitSkip => {
            e.space();
            emit_keyword(e, "SKIP");
            e.space();
            emit_keyword(e, "LOCKED");
        }
        LockWaitPolicy::LockWaitError => {
            e.space();
            emit_keyword(e, "NOWAIT");
        }
        LockWaitPolicy::LockWaitBlock | LockWaitPolicy::Undefined => {}
    }

    e.group_end();
}
