use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dangerous lock statements should be preceded by `SET idle_in_transaction_session_timeout`.
    ///
    /// A transaction holding dangerous locks that goes idle (e.g., due to application errors)
    /// will block other operations indefinitely. Setting `idle_in_transaction_session_timeout`
    /// ensures the session is terminated if it sits idle too long.
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
    pub RequireIdleInTransactionTimeout {
        version: "next",
        name: "requireIdleInTransactionTimeout",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("missing-idle-timeout")],
    }
}

impl LinterRule for RequireIdleInTransactionTimeout {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tx_state = ctx.file_context().transaction_state();
        if tx_state.has_idle_in_transaction_timeout() {
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
                    "Statement takes a dangerous lock without "<Emphasis>"idle_in_transaction_session_timeout"</Emphasis>" set."
                },
            )
            .detail(
                None,
                "Run SET idle_in_transaction_session_timeout = '...' before this statement to prevent idle transactions from holding locks.",
            ),
        ]
    }
}
