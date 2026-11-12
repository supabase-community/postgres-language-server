use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::TransactionMode;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Detects problematic transaction nesting that could lead to unexpected behavior.
    ///
    /// Transaction nesting issues occur when trying to start a transaction within an existing transaction,
    /// or trying to commit/rollback when not in a transaction. This can lead to unexpected behavior
    /// or errors in database migrations.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// BEGIN;
    /// -- Migration tools already manage transactions
    /// SELECT 1;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// SELECT 1;
    /// COMMIT; -- No transaction to commit
    /// ```
    ///
    /// In migration sections that run without a transaction, like dbmate's
    /// `-- migrate:up transaction:false`, explicit transactions are fine. The rule only reports
    /// a `BEGIN` inside a transaction, and a `COMMIT` or `ROLLBACK` outside of one.
    ///
    pub TransactionNesting {
        version: "0.15.0",
        name: "transactionNesting",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("transaction-nesting")],
    }
}

impl LinterRule for TransactionNesting {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::TransactionStmt(stmt) = &ctx.stmt() {
            if ctx.session().transaction_mode() == TransactionMode::Autocommit {
                return explicit_transaction_nesting(ctx, stmt);
            }
            match stmt.kind() {
                pgls_query::protobuf::TransactionStmtKind::TransStmtBegin
                | pgls_query::protobuf::TransactionStmtKind::TransStmtStart => {
                    if ctx.session().in_explicit_transaction() {
                        diagnostics.push(LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Nested transaction detected."
                            },
                        ).detail(None, "Starting a transaction when already in a transaction can cause issues."));
                    }
                    // Always warn about BEGIN/START since we assume we're in a transaction
                    diagnostics.push(LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Transaction already managed by migration tool."
                        },
                    ).detail(None, "Migration tools manage transactions automatically. Remove explicit transaction control.")
                    .note("Put migration statements in separate files to have them be in separate transactions."));
                }
                pgls_query::protobuf::TransactionStmtKind::TransStmtCommit
                | pgls_query::protobuf::TransactionStmtKind::TransStmtRollback => {
                    // Always warn about COMMIT/ROLLBACK since we assume we're in a transaction
                    diagnostics.push(LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Attempting to end transaction managed by migration tool."
                        },
                    ).detail(None, "Migration tools manage transactions automatically. Remove explicit transaction control.")
                    .note("Put migration statements in separate files to have them be in separate transactions."));
                }
                _ => {}
            }
        }

        diagnostics
    }
}

/// The migration tool doesn't manage a transaction, so only explicit transactions can nest.
fn explicit_transaction_nesting(
    ctx: &LinterRuleContext<TransactionNesting>,
    stmt: &pgls_query::protobuf::TransactionStmt,
) -> Vec<LinterDiagnostic> {
    let in_transaction = ctx.session().in_explicit_transaction();
    match stmt.kind() {
        pgls_query::protobuf::TransactionStmtKind::TransStmtBegin
        | pgls_query::protobuf::TransactionStmtKind::TransStmtStart
            if in_transaction =>
        {
            vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Nested transaction detected."
                    },
                )
                .detail(
                    None,
                    "Starting a transaction when already in a transaction can cause issues.",
                ),
            ]
        }
        pgls_query::protobuf::TransactionStmtKind::TransStmtCommit
        | pgls_query::protobuf::TransactionStmtKind::TransStmtRollback
            if !in_transaction =>
        {
            vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "No transaction to end."
                    },
                )
                .detail(
                    None,
                    "The migration runs without a transaction, and no transaction was started.",
                ),
            ]
        }
        _ => vec![],
    }
}
