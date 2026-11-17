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
                c.push_savepoint(None);
            }
        }
        Kind::TransStmtSavepoint => c.push_savepoint(Some(n.savepoint_name.clone())),
        Kind::TransStmtRelease => {
            if let Some(position) = c.savepoint_position(&n.savepoint_name) {
                c.savepoints.truncate(position);
                if c.savepoints.is_empty() {
                    c.end_transaction();
                }
            }
        }
        Kind::TransStmtRollbackTo => {
            if let Some(position) = c.savepoint_position(&n.savepoint_name) {
                // The savepoint stays, and a later ROLLBACK TO returns to it again.
                c.savepoints.truncate(position + 1);
                c.roll_back_to_last_savepoint();
            }
        }
        Kind::TransStmtRollback | Kind::TransStmtRollbackPrepared => {
            if !c.savepoints.is_empty() {
                c.savepoints.truncate(1);
                c.roll_back_to_last_savepoint();
                c.end_transaction();
            }
        }
        Kind::TransStmtCommit | Kind::TransStmtCommitPrepared | Kind::TransStmtPrepare => {
            c.end_transaction();
        }
        _ => {}
    }
}

impl Catalog {
    /// Remembers where the undo logs and flags are, and starts the logs at the first savepoint.
    fn push_savepoint(&mut self, name: Option<String>) {
        if self.savepoints.is_empty() {
            for journal in self.journals() {
                journal.set_recording(true);
            }
        }
        let savepoint = Savepoint {
            name,
            undo_lens: self.journals().map(|journal| journal.undo_len()),
            flags: self.flags(),
        };
        self.savepoints.push(savepoint);
    }

    /// Undoes the changes made since the last savepoint. O(changes since then).
    fn roll_back_to_last_savepoint(&mut self) {
        let Some(savepoint) = self.savepoints.last().cloned() else {
            return;
        };
        for (journal, len) in self.journals().into_iter().zip(savepoint.undo_lens) {
            journal.undo_to(len);
        }
        self.restore_flags(savepoint.flags);
    }

    /// Forgets the savepoints and the undo logs.
    fn end_transaction(&mut self) {
        self.savepoints.clear();
        for journal in self.journals() {
            journal.set_recording(false);
        }
    }

    /// The innermost savepoint called `name`.
    fn savepoint_position(&self, name: &str) -> Option<usize> {
        self.savepoints
            .iter()
            .rposition(|savepoint| savepoint.name.as_deref() == Some(name))
    }
}
