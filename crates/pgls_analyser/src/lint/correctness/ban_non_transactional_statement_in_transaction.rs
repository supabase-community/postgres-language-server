use crate::transaction_block::{non_transactional_statement, transaction_block_diagnostic};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_diagnostics::Severity;

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
        // The statements with `CONCURRENTLY` are left to `banConcurrentIndexCreationInTransaction`.
        non_transactional_statement(ctx)
            .filter(|statement| !statement.concurrently)
            .and_then(|statement| transaction_block_diagnostic(ctx, rule_category!(), statement))
            .into_iter()
            .collect()
    }
}
