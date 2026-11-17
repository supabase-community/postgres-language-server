use pgls_query::protobuf::{TransactionStmt, TransactionStmtKind as Kind};

use crate::catalog::{Catalog, Savepoint};

/// DDL is transactional: `ROLLBACK` restores the catalog of `BEGIN` or the savepoint, by undoing
/// the writes that the overlay's maps recorded since then. Models
/// [`BeginTransactionBlock`], [`DefineSavepoint`], [`RollbackToSavepoint`], [`ReleaseSavepoint`],
/// [`EndTransactionBlock`] and [`UserAbortTransactionBlock`].
///
/// [`BeginTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L3924
/// [`DefineSavepoint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4373
/// [`RollbackToSavepoint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4567
/// [`ReleaseSavepoint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4458
/// [`EndTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4044
/// [`UserAbortTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4204
pub(super) fn apply_transaction_stmt(c: &mut Catalog, n: &TransactionStmt) {
    match n.kind() {
        Kind::TransStmtBegin | Kind::TransStmtStart => {
            // A nested BEGIN only warns.
            if c.savepoints.is_empty() {
                push_savepoint(c, None);
            }
        }
        Kind::TransStmtSavepoint => push_savepoint(c, Some(n.savepoint_name.clone())),
        Kind::TransStmtRelease => {
            if let Some(position) = savepoint_position(c, &n.savepoint_name) {
                c.savepoints.truncate(position);
                if c.savepoints.is_empty() {
                    end_transaction(c);
                }
            }
        }
        Kind::TransStmtRollbackTo => {
            if let Some(position) = savepoint_position(c, &n.savepoint_name) {
                // The savepoint stays, and a later ROLLBACK TO returns to it again.
                c.savepoints.truncate(position + 1);
                roll_back_to_last(c);
            }
        }
        Kind::TransStmtRollback | Kind::TransStmtRollbackPrepared => {
            if !c.savepoints.is_empty() {
                c.savepoints.truncate(1);
                roll_back_to_last(c);
                end_transaction(c);
            }
        }
        Kind::TransStmtCommit | Kind::TransStmtCommitPrepared | Kind::TransStmtPrepare => {
            end_transaction(c);
        }
        _ => {}
    }
}

/// Remembers where the undo logs and flags are, and starts the logs at the first savepoint.
fn push_savepoint(c: &mut Catalog, name: Option<String>) {
    if c.savepoints.is_empty() {
        for journal in c.journals() {
            journal.set_recording(true);
        }
    }
    let savepoint = Savepoint {
        name,
        undo_lens: c.journals().map(|journal| journal.undo_len()),
        flags: c.flags(),
    };
    c.savepoints.push(savepoint);
}

/// Undoes the changes made since the last savepoint. O(changes since then).
fn roll_back_to_last(c: &mut Catalog) {
    let Some(savepoint) = c.savepoints.last().cloned() else {
        return;
    };
    for (journal, len) in c.journals().into_iter().zip(savepoint.undo_lens) {
        journal.undo_to(len);
    }
    c.restore_flags(savepoint.flags);
}

/// Forgets the savepoints and the undo logs.
fn end_transaction(c: &mut Catalog) {
    c.savepoints.clear();
    for journal in c.journals() {
        journal.set_recording(false);
    }
}

fn savepoint_position(c: &Catalog, name: &str) -> Option<usize> {
    c.savepoints
        .iter()
        .rposition(|savepoint| savepoint.name.as_deref() == Some(name))
}
