use crate::{
    LinterDiagnostic, LinterRule, LinterRuleContext, linter_context::is_reindex_concurrent,
};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Concurrent index operations are not allowed within a transaction.
    ///
    /// `CREATE INDEX CONCURRENTLY`, `DROP INDEX CONCURRENTLY`, `REINDEX CONCURRENTLY` and
    /// `ALTER TABLE ... DETACH PARTITION ... CONCURRENTLY` cannot be used within a transaction
    /// block, or in the body of a function. This will cause an error in Postgres.
    ///
    /// Migration tools usually run each migration in a transaction, so using `CONCURRENTLY` will fail in such tools.
    ///
    /// With `migrations.tool`, the rule knows whether the migration runs in a transaction:
    /// dbmate's `-- migrate:up transaction:false` and goose's `-- +goose NO TRANSACTION` disable it.
    /// dbmate still sends the whole section to Postgres as one query, which Postgres runs in a
    /// transaction block if it has more than one statement.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// BEGIN;
    /// CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
    /// COMMIT;
    /// ```
    ///
    pub BanConcurrentIndexCreationInTransaction {
        version: "0.15.0",
        name: "banConcurrentIndexCreationInTransaction",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Squawk("ban-concurrent-index-creation-in-transaction")],
    }
}

impl LinterRule for BanConcurrentIndexCreationInTransaction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(statement) = concurrent_statement(ctx.stmt()) else {
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

        if !ctx.in_transaction() {
            return vec![];
        }
        let diagnostic = LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    {statement}" cannot be used inside a transaction block."
                }
            ).detail(None, format!("Run {statement} outside of a transaction. Migration tools usually run in transactions, so you may need to run this statement in its own migration or manually."));
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

/// The statement, if it is one that Postgres can't run in a transaction block because of
/// `CONCURRENTLY`.
fn concurrent_statement(stmt: &pgls_query::NodeEnum) -> Option<&'static str> {
    use pgls_query::NodeEnum;
    match stmt {
        NodeEnum::IndexStmt(stmt) if stmt.concurrent => Some("CREATE INDEX CONCURRENTLY"),
        NodeEnum::DropStmt(stmt) if stmt.concurrent => Some("DROP INDEX CONCURRENTLY"),
        NodeEnum::ReindexStmt(stmt) if is_reindex_concurrent(stmt) => Some("REINDEX CONCURRENTLY"),
        NodeEnum::AlterTableStmt(stmt)
            if stmt.cmds.iter().any(|cmd| {
                matches!(
                    &cmd.node,
                    Some(NodeEnum::AlterTableCmd(cmd)) if matches!(
                        cmd.def.as_ref().and_then(|def| def.node.as_ref()),
                        Some(NodeEnum::PartitionCmd(partition)) if partition.concurrent
                    )
                )
            }) =>
        {
            Some("DETACH PARTITION CONCURRENTLY")
        }
        _ => None,
    }
}
