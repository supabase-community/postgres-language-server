use crate::transaction_block::{non_transactional_statement, transaction_block_diagnostic};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
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
    /// With `migrations.sectionMarkers`, the rule knows whether the migration runs in a transaction:
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
        non_transactional_statement(ctx)
            .filter(|statement| statement.concurrently)
            .and_then(|statement| transaction_block_diagnostic(ctx, rule_category!(), statement))
            .into_iter()
            .collect()
    }
}
