use pgls_query::{
    NodeEnum,
    protobuf::{DefElem, TransactionStmt, TransactionStmtKind, a_const},
};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str};

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
    if n.options.is_empty() {
        return;
    }

    e.space();

    for (idx, option) in n.options.iter().enumerate() {
        if idx > 0 {
            e.line(LineType::SoftOrSpace);
        }

        let def_elem = assert_node_variant!(DefElem, option);
        emit_transaction_option(e, def_elem);
    }
}

fn emit_transaction_option(e: &mut EventEmitter, def: &DefElem) {
    match def.defname.as_str() {
        "transaction_isolation" => {
            e.token(TokenKind::ISOLATION_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);

            if let Some(level) = def_elem_string(def) {
                e.space();
                emit_keyword_sequence(e, level);
            }
        }
        "transaction_read_only" => {
            if let Some(flag) = def_elem_bool(def) {
                e.token(TokenKind::READ_KW);
                e.space();
                if flag {
                    e.token(TokenKind::ONLY_KW);
                } else {
                    e.token(TokenKind::WRITE_KW);
                }
            }
        }
        "transaction_deferrable" => {
            if let Some(flag) = def_elem_bool(def) {
                if flag {
                    e.token(TokenKind::DEFERRABLE_KW);
                } else {
                    e.token(TokenKind::NOT_KW);
                    e.space();
                    e.token(TokenKind::DEFERRABLE_KW);
                }
            }
        }
        _ => {
            super::def_elem::emit_def_elem(e, def);
        }
    }
}

fn def_elem_string(def: &DefElem) -> Option<&str> {
    let arg = def.arg.as_ref()?;
    match arg.node.as_ref()? {
        NodeEnum::AConst(a_const) => match a_const.val.as_ref()? {
            a_const::Val::Sval(s) => Some(s.sval.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn def_elem_bool(def: &DefElem) -> Option<bool> {
    let arg = def.arg.as_ref()?;
    match arg.node.as_ref()? {
        NodeEnum::AConst(a_const) => match a_const.val.as_ref()? {
            a_const::Val::Boolval(v) => Some(v.boolval),
            a_const::Val::Ival(i) => Some(i.ival != 0),
            _ => None,
        },
        _ => None,
    }
}

fn emit_keyword_sequence(e: &mut EventEmitter, value: &str) {
    for (idx, part) in value.split_whitespace().enumerate() {
        if idx > 0 {
            e.space();
        }
        emit_keyword(e, &part.to_uppercase());
    }
}
