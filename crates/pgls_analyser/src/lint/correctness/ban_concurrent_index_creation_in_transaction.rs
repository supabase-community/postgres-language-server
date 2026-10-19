use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Concurrent index creation is not allowed within a transaction.
    ///
    /// `CREATE INDEX CONCURRENTLY` cannot be used within a transaction block. This will cause an error in Postgres.
    ///
    /// Migration tools usually run each migration in a transaction, so using `CREATE INDEX CONCURRENTLY` will fail in such tools.
    ///
    /// With `migrations.sectionMarkers`, the rule knows whether the migration runs in a transaction:
    /// dbmate's `-- migrate:up transaction:false` and goose's `-- +goose NO TRANSACTION` disable it.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
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
        let mut diagnostics = Vec::new();

        // Without migration sections, we don't know whether the migration tool runs the file in
        // a transaction. Migration tools usually do, so we flag concurrent indexes unless the
        // file has only this statement.
        if let pgls_query::NodeEnum::IndexStmt(stmt) = ctx.stmt()
            && stmt.concurrent
            && ctx
                .session()
                .in_transaction()
                .unwrap_or_else(|| ctx.file_context().stmt_count() > 1)
        {
            diagnostics.push(LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "CREATE INDEX CONCURRENTLY cannot be used inside a transaction block."
                    }
                ).detail(None, "Run CREATE INDEX CONCURRENTLY outside of a transaction. Migration tools usually run in transactions, so you may need to run this statement in its own migration or manually."));
        }

        diagnostics
    }
}
