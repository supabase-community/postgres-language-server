use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dangerous lock statements should be preceded by `SET statement_timeout`.
    ///
    /// Long-running statements holding locks can block other operations. Setting a
    /// `statement_timeout` ensures the statement fails rather than running indefinitely.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ADD COLUMN email TEXT;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE INDEX CONCURRENTLY users_email_idx ON users(email);
    /// ```
    ///
    pub RequireStatementTimeout {
        version: "next",
        name: "requireStatementTimeout",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("missing-statement-timeout")],
    }
}

impl LinterRule for RequireStatementTimeout {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tx_state = ctx.file_context().transaction_state();
        if tx_state.has_statement_timeout() {
            return vec![];
        }

        if !tx_state.is_dangerous_lock_stmt(ctx.stmt()) {
            return vec![];
        }

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Statement takes a dangerous lock without a "<Emphasis>"statement_timeout"</Emphasis>" set."
                },
            )
            .detail(
                None,
                "Run SET statement_timeout = '...' before this statement to prevent it from running indefinitely.",
            ),
        ]
    }
}
