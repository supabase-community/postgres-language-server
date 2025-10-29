use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Concurrent index creation is not allowed within a transaction.
    ///
    /// `CREATE INDEX CONCURRENTLY` cannot be used within a transaction block. This will cause an error in Postgres.
    ///
    /// Migration tools usually run each migration in a transaction, so using `CREATE INDEX CONCURRENTLY` will fail in such tools.
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
        version: "next",
        name: "banConcurrentIndexCreationInTransaction",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Squawk("ban-concurrent-index-creation-in-transaction")],
    }
}

impl Rule for BanConcurrentIndexCreationInTransaction {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // check if the current statement is CREATE INDEX CONCURRENTLY and there is at least one
        // other statement in the same context (indicating a transaction block)
        //
        // since our analyser assumes we're always in a transaction context, we always flag concurrent indexes
        if let pgls_query::NodeEnum::IndexStmt(stmt) = ctx.stmt() {
            if stmt.concurrent && ctx.file_context().stmt_count() > 1 {
                diagnostics.push(RuleDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "CREATE INDEX CONCURRENTLY cannot be used inside a transaction block."
                    }
                ).detail(None, "Run CREATE INDEX CONCURRENTLY outside of a transaction. Migration tools usually run in transactions, so you may need to run this statement in its own migration or manually."));
            }
        }

        diagnostics
    }
}
