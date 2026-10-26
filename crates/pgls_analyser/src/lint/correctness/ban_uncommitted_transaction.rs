use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::TransactionMode;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{TransactionStmt, TransactionStmtKind};

declare_lint_rule! {
    /// A transaction is started but never committed or rolled back.
    ///
    /// If a migration starts a transaction with `BEGIN` and never ends it, the migration may run
    /// successfully without the changes ever being applied: the session closes, and Postgres
    /// rolls the transaction back.
    ///
    /// The rule is silent in migration sections that the migration tool runs in a transaction
    /// of its own, which commits the changes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// begin;
    /// create table users (id bigint);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// begin;
    /// create table users (id bigint);
    /// commit;
    /// ```
    ///
    pub BanUncommittedTransaction {
        version: "next",
        name: "banUncommittedTransaction",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-uncommitted-transaction")],
    }
}

impl LinterRule for BanUncommittedTransaction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if ctx.session().transaction_mode() == TransactionMode::Transaction {
            return vec![];
        }

        let pgls_query::NodeEnum::TransactionStmt(stmt) = ctx.stmt() else {
            return vec![];
        };
        if !starts_transaction(stmt) {
            return vec![];
        }

        let file_context = ctx.file_context();
        let later_stmts = &file_context.stmts[file_context.previous_stmts().len() + 1..];
        let is_ended = later_stmts.iter().any(|stmt| {
            matches!(
                stmt,
                pgls_query::NodeEnum::TransactionStmt(stmt) if starts_transaction(stmt) || ends_transaction(stmt)
            )
        });
        if is_ended {
            return vec![];
        }

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Transaction is never committed or rolled back."
                },
            )
            .detail(
                None,
                "Postgres rolls back the open transaction when the session ends, so its changes are not applied.",
            )
            .note("Add a COMMIT or ROLLBACK statement to complete the transaction."),
        ]
    }
}

/// `COMMIT AND CHAIN` and `ROLLBACK AND CHAIN` immediately start a new transaction.
fn starts_transaction(stmt: &TransactionStmt) -> bool {
    match stmt.kind() {
        TransactionStmtKind::TransStmtBegin | TransactionStmtKind::TransStmtStart => true,
        TransactionStmtKind::TransStmtCommit | TransactionStmtKind::TransStmtRollback => stmt.chain,
        _ => false,
    }
}

fn ends_transaction(stmt: &TransactionStmt) -> bool {
    matches!(
        stmt.kind(),
        TransactionStmtKind::TransStmtCommit
            | TransactionStmtKind::TransStmtRollback
            | TransactionStmtKind::TransStmtPrepare
    )
}
