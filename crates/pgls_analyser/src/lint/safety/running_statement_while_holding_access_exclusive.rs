use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access.
    ///
    /// When a transaction acquires an ACCESS EXCLUSIVE lock (e.g., via ALTER TABLE), it blocks all other
    /// operations on that table, including reads. Running additional statements in the same transaction
    /// extends the duration the lock is held, potentially blocking all database access to that table.
    ///
    /// This is particularly problematic because:
    /// - The lock blocks all SELECT, INSERT, UPDATE, DELETE operations
    /// - The lock is held for the entire duration of all subsequent statements
    /// - Even simple queries like SELECT COUNT(*) can significantly extend lock time
    ///
    /// To minimize blocking, run the ALTER TABLE in its own transaction and execute
    /// other operations in separate transactions.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE authors ADD COLUMN email TEXT;
    /// SELECT COUNT(*) FROM authors;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- Run ALTER TABLE alone, other queries in separate transactions
    /// ALTER TABLE authors ADD COLUMN email TEXT;
    /// ```
    ///
    pub RunningStatementWhileHoldingAccessExclusive {
        version: "next",
        name: "runningStatementWhileHoldingAccessExclusive",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Eugene("E4")],
    }
}

impl Rule for RunningStatementWhileHoldingAccessExclusive {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check if we're currently holding an ACCESS EXCLUSIVE lock
        let tx_state = ctx.file_context().transaction_state();
        if tx_state.is_holding_access_exclusive() {
            diagnostics.push(
                RuleDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Running statement while holding ACCESS EXCLUSIVE lock."
                    },
                )
                .detail(
                    None,
                    "This blocks all access to the table for the duration of this statement.",
                )
                .note("Run this statement in a separate transaction to minimize lock duration."),
            );
        }

        diagnostics
    }
}
