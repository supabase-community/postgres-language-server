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

        if !is_dangerous_lock_stmt(ctx.stmt(), tx_state) {
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

fn is_dangerous_lock_stmt(
    stmt: &pgls_query::NodeEnum,
    tx_state: &crate::linter_context::TransactionState,
) -> bool {
    match stmt {
        pgls_query::NodeEnum::AlterTableStmt(alter_stmt) => {
            if let Some(relation) = &alter_stmt.relation {
                !tx_state.has_created_object(&relation.schemaname, &relation.relname)
            } else {
                true
            }
        }
        pgls_query::NodeEnum::IndexStmt(index_stmt) => {
            if index_stmt.concurrent {
                return false;
            }
            if let Some(relation) = &index_stmt.relation {
                !tx_state.has_created_object(&relation.schemaname, &relation.relname)
            } else {
                true
            }
        }
        _ => false,
    }
}
