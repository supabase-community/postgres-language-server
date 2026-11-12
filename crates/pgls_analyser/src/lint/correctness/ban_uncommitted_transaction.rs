use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_catalog::{TransactionMode, ends_transaction_block};
use pgls_console::markup;
use pgls_diagnostics::Severity;

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
        fix_kind: FixKind::Unsafe,
    }
}

impl LinterRule for BanUncommittedTransaction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if ctx.session().transaction_mode() == TransactionMode::Transaction {
            return vec![];
        }

        if !ctx.session().opens_transaction_block(ctx.stmt()) {
            return vec![];
        }

        // Each walk ends at the end of the transaction block that the statement opens, so the
        // walks of a file cover each statement once.
        let file_context = ctx.file_context();
        if file_context
            .later_stmts()
            .iter()
            .copied()
            .any(ends_transaction_block)
        {
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
            .note("Add a COMMIT or ROLLBACK statement to complete the transaction.")
            .maybe_fix(
                file_context
                    .append_statement("commit;")
                    .map(|(at, text)| RuleFix::new("Add COMMIT at the end").insert(at, text)),
            ),
        ]
    }
}
