use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::{NodeEnum, protobuf};
use pgls_query_ext::options::{def_elem_bool, find_def_elem, is_reindex_concurrent};

declare_lint_rule! {
    /// Statements that Postgres can't run inside a transaction block.
    ///
    /// `VACUUM`, `CREATE DATABASE`, `DROP DATABASE`, `ALTER DATABASE ... SET TABLESPACE`,
    /// `CREATE TABLESPACE`, `DROP TABLESPACE`, `ALTER SYSTEM`, `CLUSTER` without a table or on a
    /// partitioned table, `REINDEX SCHEMA`, `REINDEX SYSTEM`, `REINDEX DATABASE`, `REINDEX TABLE`
    /// on a partitioned table, `REINDEX INDEX` on a partitioned index, `DISCARD ALL`,
    /// `COMMIT PREPARED`, `ROLLBACK PREPARED`, `CREATE SUBSCRIPTION` that creates a replication
    /// slot, and `ALTER SUBSCRIPTION ... REFRESH PUBLICATION` fail in a transaction block and in
    /// the body of a function. The statements with `CONCURRENTLY` are covered by
    /// `banConcurrentIndexCreationInTransaction`.
    ///
    /// The rule only reports statements that surely run in a transaction block: between `BEGIN`
    /// and `COMMIT`, in a function body, or in a migration section that runs in a transaction
    /// (see `migrations.sectionMarkers`). Without section markers, we don't know whether the
    /// migration tool runs the file in a transaction, so the other statements aren't reported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// begin;
    /// vacuum users;
    /// commit;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// vacuum users;
    /// ```
    ///
    pub BanNonTransactionalStatementInTransaction {
        version: "next",
        name: "banNonTransactionalStatementInTransaction",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for BanNonTransactionalStatementInTransaction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(statement) = non_transactional_statement(ctx) else {
            return vec![];
        };

        if ctx.function().is_some() {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        {statement}" cannot be executed from a function."
                    },
                )
                .detail(
                    None,
                    "The statements of a function run in the transaction of the statement that calls it. Run the statement outside of the function.",
                ),
            ];
        }

        if !ctx.knows_transaction() || !ctx.in_transaction() {
            return vec![];
        }
        let diagnostic = LinterDiagnostic::new(
            rule_category!(),
            None,
            markup! {
                {statement}" cannot run inside a transaction block."
            },
        )
        .detail(
            None,
            format!("Run {statement} outside of a transaction, e.g. in a migration of its own that runs without a transaction."),
        );
        let diagnostic = if ctx.session().in_implicit_block()
            && ctx.session().transaction_depth() == 0
        {
            diagnostic.note("The migration tool sends the statements to Postgres as one query, and Postgres runs a query with more than one statement in a transaction block. Put the statement in a section of its own.")
        } else {
            diagnostic
        };
        vec![diagnostic]
    }
}

/// The statement, if it is one that Postgres refuses to run in a transaction block. These are
/// the callers of [`PreventInTransactionBlock`] that we can recognize without the database,
/// except for the `CONCURRENTLY` ones.
///
/// [`PreventInTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L3648
fn non_transactional_statement(
    ctx: &LinterRuleContext<BanNonTransactionalStatementInTransaction>,
) -> Option<&'static str> {
    use protobuf::{AlterSubscriptionType, DiscardMode, ReindexObjectType, TransactionStmtKind};

    let is_partitioned_table = |relation: &protobuf::RangeVar| {
        ctx.catalog()
            .is_partitioned_table(relation, ctx.session().search_path())
    };
    let is_partitioned_index = |index: &protobuf::RangeVar| {
        ctx.catalog()
            .is_partitioned_index_of(index, ctx.session().search_path())
    };
    match ctx.stmt() {
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L523
        NodeEnum::VacuumStmt(stmt) if stmt.is_vacuumcmd => Some("VACUUM"),
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L715
        NodeEnum::CreateTableSpaceStmt(_) => Some("CREATE TABLESPACE"),
        NodeEnum::DropTableSpaceStmt(_) => Some("DROP TABLESPACE"),
        NodeEnum::CreatedbStmt(_) => Some("CREATE DATABASE"),
        NodeEnum::DropdbStmt(_) => Some("DROP DATABASE"),
        NodeEnum::AlterSystemStmt(_) => Some("ALTER SYSTEM"),
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/dbcommands.c#L2439
        NodeEnum::AlterDatabaseStmt(stmt)
            if find_def_elem(&stmt.options, "tablespace").is_some() =>
        {
            Some("ALTER DATABASE SET TABLESPACE")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/cluster.c#L210
        NodeEnum::ClusterStmt(stmt) => match &stmt.relation {
            None => Some("CLUSTER"),
            Some(relation) if !stmt.indexname.is_empty() && is_partitioned_table(relation) => {
                Some("CLUSTER")
            }
            Some(_) => None,
        },
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L2919
        NodeEnum::ReindexStmt(stmt) if !is_reindex_concurrent(stmt) => match stmt.kind() {
            ReindexObjectType::ReindexObjectSchema => Some("REINDEX SCHEMA"),
            ReindexObjectType::ReindexObjectSystem => Some("REINDEX SYSTEM"),
            ReindexObjectType::ReindexObjectDatabase => Some("REINDEX DATABASE"),
            // `ReindexPartitions`, for a partitioned table or index:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L3392
            ReindexObjectType::ReindexObjectTable
                if stmt.relation.as_ref().is_some_and(is_partitioned_table) =>
            {
                Some("REINDEX TABLE")
            }
            ReindexObjectType::ReindexObjectIndex
                if stmt.relation.as_ref().is_some_and(is_partitioned_index) =>
            {
                Some("REINDEX INDEX")
            }
            _ => None,
        },
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/discard.c#L66
        NodeEnum::DiscardStmt(stmt) if stmt.target() == DiscardMode::DiscardAll => {
            Some("DISCARD ALL")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/utility.c#L652
        NodeEnum::TransactionStmt(stmt) => match stmt.kind() {
            TransactionStmtKind::TransStmtCommitPrepared => Some("COMMIT PREPARED"),
            TransactionStmtKind::TransStmtRollbackPrepared => Some("ROLLBACK PREPARED"),
            _ => None,
        },
        // `create_slot` is on by default, unless `connect = false`. Postgres rejects an invalid
        // value of either before it checks for a transaction block.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/subscriptioncmds.c#L602
        NodeEnum::CreateSubscriptionStmt(stmt) => {
            let option = |name| find_def_elem(&stmt.options, name).map(def_elem_bool);
            let connect = option("connect").unwrap_or(Some(true));
            let create_slot = option("create_slot").unwrap_or(connect);
            (connect.is_some() && create_slot == Some(true))
                .then_some("CREATE SUBSCRIPTION ... WITH (create_slot = true)")
        }
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/subscriptioncmds.c#L1535
        NodeEnum::AlterSubscriptionStmt(stmt)
            if stmt.kind() == AlterSubscriptionType::AlterSubscriptionRefresh =>
        {
            Some("ALTER SUBSCRIPTION ... REFRESH")
        }
        _ => None,
    }
}
