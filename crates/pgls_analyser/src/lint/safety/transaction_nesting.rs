use pgls_analyse::{
    AnalysedFileContext, Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule,
};
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
    pub TransactionNesting {
        version: "next",
        name: "transactionNesting",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("transaction-nesting")],
    }
}

impl Rule for TransactionNesting {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::TransactionStmt(stmt) = &ctx.stmt() {
            match stmt.kind() {
                pgls_query::protobuf::TransactionStmtKind::TransStmtBegin
                | pgls_query::protobuf::TransactionStmtKind::TransStmtStart => {
                    // Check if there's already a BEGIN in previous statements
                    if has_transaction_start_before(ctx.file_context()) {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Nested transaction detected."
                            },
                        ).detail(None, "Starting a transaction when already in a transaction can cause issues."));
                    }
                    // Always warn about BEGIN/START since we assume we're in a transaction
                    diagnostics.push(RuleDiagnostic::new(
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
                    diagnostics.push(RuleDiagnostic::new(
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

fn has_transaction_start_before(file_context: &AnalysedFileContext) -> bool {
    for stmt in file_context.previous_stmts() {
        if let pgls_query::NodeEnum::TransactionStmt(tx_stmt) = stmt {
            match tx_stmt.kind() {
                pgls_query::protobuf::TransactionStmtKind::TransStmtBegin
                | pgls_query::protobuf::TransactionStmtKind::TransStmtStart => return true,
                pgls_query::protobuf::TransactionStmtKind::TransStmtCommit
                | pgls_query::protobuf::TransactionStmtKind::TransStmtRollback => return false,
                _ => {}
            }
        }
    }
    false
}
