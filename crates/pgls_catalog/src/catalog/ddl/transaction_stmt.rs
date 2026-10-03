use pgls_query::protobuf::{TransactionStmt, TransactionStmtKind as Kind};

use crate::catalog::{Catalog, Savepoint};

/// DDL is transactional: `ROLLBACK` restores the catalog of `BEGIN` or the savepoint. Models
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
            }
        }
        Kind::TransStmtRollbackTo => {
            if let Some(position) = savepoint_position(c, &n.savepoint_name) {
                let savepoints = c.savepoints[..=position].to_vec();
                *c = *savepoints[position].catalog.clone();
                c.savepoints = savepoints;
            }
        }
        Kind::TransStmtRollback | Kind::TransStmtRollbackPrepared => {
            if let Some(savepoint) = c.savepoints.first().cloned() {
                *c = *savepoint.catalog;
            }
        }
        Kind::TransStmtCommit | Kind::TransStmtCommitPrepared | Kind::TransStmtPrepare => {
            c.savepoints.clear();
        }
        _ => {}
    }
}

fn push_savepoint(c: &mut Catalog, name: Option<String>) {
    let mut catalog = c.clone();
    catalog.savepoints.clear();
    c.savepoints.push(Savepoint {
        name,
        catalog: Box::new(catalog),
    });
}

fn savepoint_position(c: &Catalog, name: &str) -> Option<usize> {
    c.savepoints
        .iter()
        .rposition(|savepoint| savepoint.name.as_deref() == Some(name))
}
