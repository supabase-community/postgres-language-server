//! Transaction blocks: the statements that end one, and the statements that Postgres refuses to
//! run in one.

use pgls_query::{
    NodeEnum,
    protobuf::{self, AlterSubscriptionType, DiscardMode, ReindexObjectType, TransactionStmtKind},
};
use pgls_query_ext::utils::{alter_table_cmds, detaches_concurrently};

use super::locks::is_reindex_concurrent;
use crate::Catalog;

/// A statement that Postgres can't run in a transaction block, see
/// [`prevents_transaction_block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonTransactional {
    /// The statement, like `VACUUM` or `CREATE INDEX CONCURRENTLY`.
    pub statement: &'static str,
    /// Whether the statement can't run in a transaction block because of `CONCURRENTLY`, so it
    /// runs in a transaction block without it.
    pub concurrently: bool,
}

impl NonTransactional {
    fn new(statement: &'static str) -> Self {
        Self {
            statement,
            concurrently: false,
        }
    }

    fn concurrently(statement: &'static str) -> Self {
        Self {
            statement,
            concurrently: true,
        }
    }
}

/// The statement, if Postgres refuses to run it in a transaction block or from a function. These
/// are the callers of [`PreventInTransactionBlock`] that we can recognize without the state of
/// the database. `catalog` and `search_path` tell whether a table is partitioned.
///
/// Not covered, because whether they fail depends on state we don't have: `DROP SUBSCRIPTION`
/// (only with a replication slot), `ALTER SUBSCRIPTION ... SET/ADD/DROP PUBLICATION` with
/// refresh and `SET (failover/two_phase)`, `REINDEX INDEX` of a partitioned index, and the
/// walsender replication commands.
///
/// [`PreventInTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L3648
pub fn prevents_transaction_block(
    stmt: &NodeEnum,
    catalog: &Catalog,
    search_path: &[String],
) -> Option<NonTransactional> {
    let is_partitioned_table =
        |relation: &protobuf::RangeVar| catalog.is_partitioned_table(relation, search_path);
    let statement = match stmt {
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L1465
        NodeEnum::IndexStmt(stmt) if stmt.concurrent => {
            NonTransactional::concurrently("CREATE INDEX CONCURRENTLY")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L2017
        NodeEnum::DropStmt(stmt) if stmt.concurrent => {
            NonTransactional::concurrently("DROP INDEX CONCURRENTLY")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L2870
        NodeEnum::ReindexStmt(stmt) if is_reindex_concurrent(stmt) => {
            NonTransactional::concurrently("REINDEX CONCURRENTLY")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L1291
        NodeEnum::AlterTableStmt(stmt) if alter_table_cmds(stmt).any(detaches_concurrently) => {
            NonTransactional::concurrently("DETACH PARTITION CONCURRENTLY")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L523
        NodeEnum::VacuumStmt(stmt) if stmt.is_vacuumcmd => NonTransactional::new("VACUUM"),
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L715
        NodeEnum::CreateTableSpaceStmt(_) => NonTransactional::new("CREATE TABLESPACE"),
        NodeEnum::DropTableSpaceStmt(_) => NonTransactional::new("DROP TABLESPACE"),
        NodeEnum::CreatedbStmt(_) => NonTransactional::new("CREATE DATABASE"),
        NodeEnum::DropdbStmt(_) => NonTransactional::new("DROP DATABASE"),
        NodeEnum::AlterSystemStmt(_) => NonTransactional::new("ALTER SYSTEM"),
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/dbcommands.c#L2439
        NodeEnum::AlterDatabaseStmt(stmt)
            if stmt.options.iter().any(|option| {
                matches!(&option.node, Some(NodeEnum::DefElem(def)) if def.defname == "tablespace")
            }) =>
        {
            NonTransactional::new("ALTER DATABASE SET TABLESPACE")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/cluster.c#L210
        NodeEnum::ClusterStmt(stmt) => match &stmt.relation {
            None => NonTransactional::new("CLUSTER"),
            Some(relation) if !stmt.indexname.is_empty() && is_partitioned_table(relation) => {
                NonTransactional::new("CLUSTER")
            }
            Some(_) => return None,
        },
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L2919
        NodeEnum::ReindexStmt(stmt) => match stmt.kind() {
            ReindexObjectType::ReindexObjectSchema => NonTransactional::new("REINDEX SCHEMA"),
            ReindexObjectType::ReindexObjectSystem => NonTransactional::new("REINDEX SYSTEM"),
            ReindexObjectType::ReindexObjectDatabase => NonTransactional::new("REINDEX DATABASE"),
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L3392
            ReindexObjectType::ReindexObjectTable
                if stmt
                    .relation
                    .as_ref()
                    .is_some_and(is_partitioned_table) =>
            {
                NonTransactional::new("REINDEX TABLE")
            }
            _ => return None,
        },
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/discard.c#L66
        NodeEnum::DiscardStmt(stmt) if stmt.target() == DiscardMode::DiscardAll => {
            NonTransactional::new("DISCARD ALL")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L652
        NodeEnum::TransactionStmt(stmt) => match stmt.kind() {
            TransactionStmtKind::TransStmtCommitPrepared => NonTransactional::new("COMMIT PREPARED"),
            TransactionStmtKind::TransStmtRollbackPrepared => {
                NonTransactional::new("ROLLBACK PREPARED")
            }
            _ => return None,
        },
        // `create_slot` is on by default, unless `connect = false`.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/subscriptioncmds.c#L602
        NodeEnum::CreateSubscriptionStmt(stmt) => {
            let create_slot = bool_option(&stmt.options, "create_slot")
                .unwrap_or_else(|| bool_option(&stmt.options, "connect").unwrap_or(true));
            if !create_slot {
                return None;
            }
            NonTransactional::new("CREATE SUBSCRIPTION ... WITH (create_slot = true)")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/subscriptioncmds.c#L1535
        NodeEnum::AlterSubscriptionStmt(stmt)
            if stmt.kind() == AlterSubscriptionType::AlterSubscriptionRefresh =>
        {
            NonTransactional::new("ALTER SUBSCRIPTION ... REFRESH")
        }
        _ => return None,
    };
    Some(statement)
}

/// Whether the statement ends a transaction block that `BEGIN` opened: `COMMIT`, `ROLLBACK` and
/// `PREPARE TRANSACTION`, also with `AND CHAIN`, which opens the next one right away. Port of
/// the `TBLOCK_INPROGRESS` cases of [`EndTransactionBlock`], [`UserAbortTransactionBlock`] and
/// [`PrepareTransactionBlock`]. A `BEGIN` or a savepoint command in a transaction block doesn't
/// end it.
///
/// [`EndTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4044
/// [`UserAbortTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L4204
/// [`PrepareTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L3992
pub fn ends_transaction_block(stmt: &NodeEnum) -> bool {
    matches!(
        stmt,
        NodeEnum::TransactionStmt(stmt) if matches!(
            stmt.kind(),
            TransactionStmtKind::TransStmtCommit
                | TransactionStmtKind::TransStmtRollback
                | TransactionStmtKind::TransStmtPrepare
        )
    )
}

/// The value of a boolean option, like [`defGetBoolean`], if it is set. An option without a
/// value is on.
///
/// [`defGetBoolean`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L94
fn bool_option(options: &[protobuf::Node], name: &str) -> Option<bool> {
    let def = options.iter().find_map(|option| match &option.node {
        Some(NodeEnum::DefElem(def)) if def.defname == name => Some(def),
        _ => None,
    })?;
    Some(match def.arg.as_ref().and_then(|arg| arg.node.as_ref()) {
        None => true,
        Some(NodeEnum::Integer(value)) => value.ival != 0,
        Some(NodeEnum::Boolean(value)) => value.boolval,
        Some(NodeEnum::String(value)) => !["false", "off", "0"]
            .iter()
            .any(|off| value.sval.eq_ignore_ascii_case(off)),
        Some(_) => true,
    })
}
