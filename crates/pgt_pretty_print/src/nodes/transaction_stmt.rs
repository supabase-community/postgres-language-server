use pgt_query::protobuf::{TransactionStmt, TransactionStmtKind};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

use super::string::{emit_identifier_maybe_quoted, emit_single_quoted_str};

pub(super) fn emit_transaction_stmt(e: &mut EventEmitter, n: &TransactionStmt) {
    e.group_start(GroupKind::TransactionStmt);

    let kind = n.kind();

    match kind {
        TransactionStmtKind::TransStmtBegin => {
            e.token(TokenKind::BEGIN_KW);
            emit_transaction_options(e, n);
        }
        TransactionStmtKind::TransStmtStart => {
            e.token(TokenKind::START_KW);
            e.space();
            e.token(TokenKind::TRANSACTION_KW);
            emit_transaction_options(e, n);
        }
        TransactionStmtKind::TransStmtCommit => {
            e.token(TokenKind::COMMIT_KW);
            if n.chain {
                e.space();
                e.token(TokenKind::AND_KW);
                e.space();
                e.token(TokenKind::CHAIN_KW);
            }
        }
        TransactionStmtKind::TransStmtRollback => {
            e.token(TokenKind::ROLLBACK_KW);
            if n.chain {
                e.space();
                e.token(TokenKind::AND_KW);
                e.space();
                e.token(TokenKind::CHAIN_KW);
            }
        }
        TransactionStmtKind::TransStmtSavepoint => {
            e.token(TokenKind::SAVEPOINT_KW);
            if !n.savepoint_name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &n.savepoint_name);
            }
        }
        TransactionStmtKind::TransStmtRelease => {
            e.token(TokenKind::RELEASE_KW);
            if !n.savepoint_name.is_empty() {
                e.space();
                e.token(TokenKind::SAVEPOINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.savepoint_name);
            }
        }
        TransactionStmtKind::TransStmtRollbackTo => {
            e.token(TokenKind::ROLLBACK_KW);
            e.space();
            e.token(TokenKind::TO_KW);
            if !n.savepoint_name.is_empty() {
                e.space();
                e.token(TokenKind::SAVEPOINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.savepoint_name);
            }
        }
        TransactionStmtKind::TransStmtPrepare => {
            e.token(TokenKind::PREPARE_KW);
            e.space();
            e.token(TokenKind::TRANSACTION_KW);
            if !n.gid.is_empty() {
                e.space();
                emit_single_quoted_str(e, &n.gid);
            }
        }
        TransactionStmtKind::TransStmtCommitPrepared => {
            e.token(TokenKind::COMMIT_KW);
            e.space();
            e.token(TokenKind::PREPARED_KW);
            if !n.gid.is_empty() {
                e.space();
                emit_single_quoted_str(e, &n.gid);
            }
        }
        TransactionStmtKind::TransStmtRollbackPrepared => {
            e.token(TokenKind::ROLLBACK_KW);
            e.space();
            e.token(TokenKind::PREPARED_KW);
            if !n.gid.is_empty() {
                e.space();
                emit_single_quoted_str(e, &n.gid);
            }
        }
        TransactionStmtKind::Undefined => {}
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_transaction_options(e: &mut EventEmitter, n: &TransactionStmt) {
    if !n.options.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.options, super::emit_node);
    }
}
