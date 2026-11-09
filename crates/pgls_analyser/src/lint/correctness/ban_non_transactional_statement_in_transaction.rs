use crate::{
    LinterDiagnostic, LinterRule, LinterRuleContext, linter_context::is_reindex_concurrent,
};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::{NodeEnum, protobuf};

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
            if stmt.options.iter().any(|option| {
                matches!(&option.node, Some(NodeEnum::DefElem(def)) if def.defname == "tablespace")
            }) =>
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
                if stmt
                    .relation
                    .as_ref()
                    .is_some_and(is_partitioned_table) =>
            {
                Some("REINDEX TABLE")
            }
            ReindexObjectType::ReindexObjectIndex
                if stmt
                    .relation
                    .as_ref()
                    .is_some_and(is_partitioned_index) =>
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
        // `create_slot` is on by default, unless `connect = false`.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/subscriptioncmds.c#L602
        NodeEnum::CreateSubscriptionStmt(stmt) => {
            let create_slot = option(&stmt.options, "create_slot")
                .unwrap_or_else(|| option(&stmt.options, "connect").unwrap_or(true));
            create_slot.then_some("CREATE SUBSCRIPTION ... WITH (create_slot = true)")
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

/// The value of a boolean option, like [`defGetBoolean`], if it is set. An option without a
/// value is on.
///
/// [`defGetBoolean`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L94
fn option(options: &[protobuf::Node], name: &str) -> Option<bool> {
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
