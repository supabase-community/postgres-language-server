use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer statements with guards for robustness in migrations.
    ///
    /// When running migrations outside of transactions (e.g., CREATE INDEX CONCURRENTLY),
    /// statements should be made robust by using guards like IF NOT EXISTS or IF EXISTS.
    /// This allows migrations to be safely re-run if they fail partway through.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// DROP INDEX CONCURRENTLY users_email_idx;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
    /// ```
    ///
    /// ```sql
    /// DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
    /// ```
    ///
    pub PreferRobustStmts {
        version: "next",
        name: "preferRobustStmts",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-robust-stmts")],
    }
}

impl Rule for PreferRobustStmts {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Since we assume we're always in a transaction, we only check for
        // statements that explicitly run outside transactions
        match &ctx.stmt() {
            pgls_query::NodeEnum::IndexStmt(stmt) => {
                // Concurrent index creation runs outside transaction
                if stmt.concurrent {
                    // Check for unnamed index
                    if stmt.idxname.is_empty() {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Concurrent index should have an explicit name."
                            },
                        ).detail(None, "Use an explicit name for a concurrently created index to make migrations more robust."));
                    }
                    // Check for IF NOT EXISTS
                    if !stmt.if_not_exists {
                        diagnostics.push(
                            RuleDiagnostic::new(
                                rule_category!(),
                                None,
                                markup! {
                                    "Concurrent index creation should use IF NOT EXISTS."
                                },
                            )
                            .detail(
                                None,
                                "Add IF NOT EXISTS to make the migration re-runnable if it fails.",
                            ),
                        );
                    }
                }
            }
            pgls_query::NodeEnum::DropStmt(stmt) => {
                // Concurrent drop runs outside transaction
                if stmt.concurrent && !stmt.missing_ok {
                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Concurrent drop should use IF EXISTS."
                            },
                        )
                        .detail(
                            None,
                            "Add IF EXISTS to make the migration re-runnable if it fails.",
                        ),
                    );
                }
            }
            _ => {}
        }

        diagnostics
    }
}
