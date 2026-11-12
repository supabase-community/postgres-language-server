//! The table locks that statements take, ported from Postgres.

use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ConstrType, RangeVar},
};
use pgls_query_ext::utils::{alter_table_cmds, cmd_def};

/// A table lock mode, from the weakest to the strongest, like [`lockdefs.h`]. Each mode
/// conflicts with the modes listed in the [docs].
///
/// [`lockdefs.h`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/storage/lockdefs.h#L36
/// [docs]: https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-TABLES
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockMode {
    AccessShare,
    RowShare,
    RowExclusive,
    ShareUpdateExclusive,
    Share,
    ShareRowExclusive,
    Exclusive,
    AccessExclusive,
}

impl LockMode {
    /// The name of the mode, like `ACCESS EXCLUSIVE`.
    pub fn name(self) -> &'static str {
        match self {
            LockMode::AccessShare => "ACCESS SHARE",
            LockMode::RowShare => "ROW SHARE",
            LockMode::RowExclusive => "ROW EXCLUSIVE",
            LockMode::ShareUpdateExclusive => "SHARE UPDATE EXCLUSIVE",
            LockMode::Share => "SHARE",
            LockMode::ShareRowExclusive => "SHARE ROW EXCLUSIVE",
            LockMode::Exclusive => "EXCLUSIVE",
            LockMode::AccessExclusive => "ACCESS EXCLUSIVE",
        }
    }

    /// Whether the mode conflicts with the `ROW EXCLUSIVE` lock of `INSERT`, `UPDATE` and
    /// `DELETE`. A statement that waits for such a lock also blocks the writes queued behind it.
    pub fn blocks_writes(self) -> bool {
        self >= LockMode::Share
    }

    /// Whether the mode conflicts with the `ACCESS SHARE` lock of `SELECT`.
    pub fn blocks_reads(self) -> bool {
        self == LockMode::AccessExclusive
    }
}

/// The lock that an `ALTER TABLE` takes on its table: the strongest lock of its subcommands, at
/// least `SHARE UPDATE EXCLUSIVE`. Port of [`AlterTableGetLockLevel`], which is the same for the
/// subcommands of Postgres 15 to 18.
///
/// [`AlterTableGetLockLevel`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L4617
pub fn alter_table_lock(stmt: &AlterTableStmt) -> LockMode {
    alter_table_cmds(stmt)
        .map(alter_table_cmd_lock)
        .fold(LockMode::ShareUpdateExclusive, LockMode::max)
}

/// The lock that an `ALTER TABLE` subcommand needs on the table, see [`alter_table_lock`].
pub fn alter_table_cmd_lock(cmd: &AlterTableCmd) -> LockMode {
    use AlterTableType as At;
    match cmd.subtype() {
        // These affect write operations only.
        At::AtEnableTrig
        | At::AtEnableAlwaysTrig
        | At::AtEnableReplicaTrig
        | At::AtEnableTrigAll
        | At::AtEnableTrigUser
        | At::AtDisableTrig
        | At::AtDisableTrigAll
        | At::AtDisableTrigUser => LockMode::ShareRowExclusive,
        At::AtAddConstraint | At::AtReAddConstraint | At::AtReAddDomainConstraint => {
            match cmd_def(cmd) {
                // A foreign key adds triggers to both tables, like `CREATE TRIGGER`.
                Some(NodeEnum::Constraint(constraint))
                    if constraint.contype() == ConstrType::ConstrForeign =>
                {
                    LockMode::ShareRowExclusive
                }
                _ => LockMode::AccessExclusive,
            }
        }
        // These change strategies for performance and maintenance, but not the results of reads
        // and writes.
        At::AtSetStatistics
        | At::AtClusterOn
        | At::AtDropCluster
        | At::AtSetOptions
        | At::AtResetOptions
        | At::AtValidateConstraint
        | At::AtAttachPartition
        | At::AtDetachPartitionFinalize => LockMode::ShareUpdateExclusive,
        At::AtSetRelOptions | At::AtResetRelOptions => rel_options_lock(cmd),
        At::AtDetachPartition => match cmd_def(cmd) {
            Some(NodeEnum::PartitionCmd(partition)) if partition.concurrent => {
                LockMode::ShareUpdateExclusive
            }
            _ => LockMode::AccessExclusive,
        },
        // Everything else may rewrite the table or change what reads see. The subcommands that
        // only Postgres itself creates, like `AT_ReAddIndex`, aren't in the switch of
        // `AlterTableGetLockLevel` and never come from the parser.
        At::AtAddColumn
        | At::AtAddColumnToView
        | At::AtColumnDefault
        | At::AtCookedColumnDefault
        | At::AtDropNotNull
        | At::AtSetNotNull
        | At::AtSetExpression
        | At::AtDropExpression
        | At::AtSetStorage
        | At::AtSetCompression
        | At::AtDropColumn
        | At::AtAddIndex
        | At::AtReAddIndex
        | At::AtAlterConstraint
        | At::AtAddIndexConstraint
        | At::AtDropConstraint
        | At::AtReAddComment
        | At::AtAlterColumnType
        | At::AtAlterColumnGenericOptions
        | At::AtChangeOwner
        | At::AtSetLogged
        | At::AtSetUnLogged
        | At::AtDropOids
        | At::AtSetAccessMethod
        | At::AtSetTableSpace
        | At::AtReplaceRelOptions
        | At::AtEnableRule
        | At::AtEnableAlwaysRule
        | At::AtEnableReplicaRule
        | At::AtDisableRule
        | At::AtAddInherit
        | At::AtDropInherit
        | At::AtAddOf
        | At::AtDropOf
        | At::AtReplicaIdentity
        | At::AtEnableRowSecurity
        | At::AtDisableRowSecurity
        | At::AtForceRowSecurity
        | At::AtNoForceRowSecurity
        | At::AtGenericOptions
        | At::AtAddIdentity
        | At::AtSetIdentity
        | At::AtDropIdentity
        | At::AtReAddStatistics
        | At::Undefined => LockMode::AccessExclusive,
    }
}

/// The lock of `SET (...)` and `RESET (...)`: the strongest lock of the options, and `ACCESS
/// EXCLUSIVE` without options. Port of [`AlterTableGetRelOptionsLockLevel`] with the lock
/// modes of the built-in options in [`reloptions.c`], which are the same in Postgres 15 to 18.
/// A misspelled option adds no lock, like in Postgres, which rejects it later. An option that an
/// extension registers has the lock mode the extension gives it, which we don't know, so it adds
/// no lock either.
///
/// [`AlterTableGetRelOptionsLockLevel`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/common/reloptions.c#L2144
/// [`reloptions.c`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/common/reloptions.c#L94
fn rel_options_lock(cmd: &AlterTableCmd) -> LockMode {
    let Some(NodeEnum::List(options)) = cmd_def(cmd) else {
        return LockMode::AccessExclusive;
    };
    if options.items.is_empty() {
        return LockMode::AccessExclusive;
    }
    options
        .items
        .iter()
        .filter_map(|option| match &option.node {
            Some(NodeEnum::DefElem(def)) => rel_option_lock(&def.defname),
            _ => None,
        })
        // `NoLock` in Postgres, which is weaker than the minimum of `alter_table_lock`.
        .fold(LockMode::AccessShare, LockMode::max)
}

/// The lock mode of a built-in storage parameter.
fn rel_option_lock(name: &str) -> Option<LockMode> {
    const ACCESS_EXCLUSIVE: &[&str] = &[
        "autosummarize",
        "buffering",
        "check_option",
        "fastupdate",
        "gin_pending_list_limit",
        "pages_per_range",
        "security_barrier",
        "security_invoker",
        "user_catalog_table",
    ];
    const SHARE_UPDATE_EXCLUSIVE: &[&str] = &[
        "autovacuum_analyze_scale_factor",
        "autovacuum_analyze_threshold",
        "autovacuum_enabled",
        "autovacuum_freeze_max_age",
        "autovacuum_freeze_min_age",
        "autovacuum_freeze_table_age",
        "autovacuum_multixact_freeze_max_age",
        "autovacuum_multixact_freeze_min_age",
        "autovacuum_multixact_freeze_table_age",
        "autovacuum_vacuum_cost_delay",
        "autovacuum_vacuum_cost_limit",
        "autovacuum_vacuum_insert_scale_factor",
        "autovacuum_vacuum_insert_threshold",
        "autovacuum_vacuum_max_threshold",
        "autovacuum_vacuum_scale_factor",
        "autovacuum_vacuum_threshold",
        "deduplicate_items",
        "effective_io_concurrency",
        "fillfactor",
        "log_autovacuum_min_duration",
        "maintenance_io_concurrency",
        "n_distinct",
        "n_distinct_inherited",
        "parallel_workers",
        "random_page_cost",
        "seq_page_cost",
        "toast_tuple_target",
        "vacuum_cleanup_index_scale_factor",
        "vacuum_index_cleanup",
        "vacuum_max_eager_freeze_failure_rate",
        "vacuum_truncate",
    ];
    if ACCESS_EXCLUSIVE.contains(&name) {
        Some(LockMode::AccessExclusive)
    } else if SHARE_UPDATE_EXCLUSIVE.contains(&name) {
        Some(LockMode::ShareUpdateExclusive)
    } else {
        None
    }
}

/// A lock that a statement takes on a table, see [`crate::Session::table_locks`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableLock {
    pub mode: LockMode,
    pub schema: String,
    pub name: String,
}

/// The locks that a statement takes on the relations it names, for the statements whose locks
/// we know: `ALTER TABLE` and `CREATE INDEX`.
///
/// `ALTER TABLE ... ATTACH PARTITION` also locks the partition `ACCESS EXCLUSIVE`, and `DETACH
/// PARTITION` the partition like the table, see [`ATExecAttachPartition`] and
/// [`ATExecDetachPartition`]. `ADD CONSTRAINT ... FOREIGN KEY` locks the referenced table
/// `SHARE ROW EXCLUSIVE`, see [`ATAddForeignKeyConstraint`]. `CREATE INDEX` takes `SHARE`, and `SHARE UPDATE EXCLUSIVE` with
/// `CONCURRENTLY`, see [`ProcessUtilitySlow`].
///
/// [`ATExecAttachPartition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L20297
/// [`ATAddForeignKeyConstraint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L10111
/// [`ATExecDetachPartition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L20959
/// [`ProcessUtilitySlow`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L1477
pub fn relation_locks(stmt: &NodeEnum) -> Vec<(LockMode, &RangeVar)> {
    match stmt {
        NodeEnum::AlterTableStmt(stmt) => {
            let others = alter_table_cmds(stmt).filter_map(|cmd| match cmd_def(cmd)? {
                NodeEnum::PartitionCmd(partition) => {
                    let mode = match cmd.subtype() {
                        AlterTableType::AtAttachPartition => LockMode::AccessExclusive,
                        AlterTableType::AtDetachPartition if partition.concurrent => {
                            LockMode::ShareUpdateExclusive
                        }
                        AlterTableType::AtDetachPartition => LockMode::AccessExclusive,
                        _ => return None,
                    };
                    Some((mode, partition.name.as_ref()?))
                }
                NodeEnum::Constraint(constraint)
                    if cmd.subtype() == AlterTableType::AtAddConstraint
                        && constraint.contype() == ConstrType::ConstrForeign =>
                {
                    Some((LockMode::ShareRowExclusive, constraint.pktable.as_ref()?))
                }
                _ => None,
            });
            stmt.relation
                .as_ref()
                .map(|relation| (alter_table_lock(stmt), relation))
                .into_iter()
                .chain(others)
                .collect()
        }
        NodeEnum::IndexStmt(stmt) => {
            let mode = if stmt.concurrent {
                LockMode::ShareUpdateExclusive
            } else {
                LockMode::Share
            };
            stmt.relation
                .as_ref()
                .map(|relation| (mode, relation))
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}
