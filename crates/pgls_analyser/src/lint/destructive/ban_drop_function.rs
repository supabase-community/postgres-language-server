use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a function, procedure, routine, or aggregate may break existing clients.
    ///
    /// Calls from clients that still use it fail with `42883 undefined_function`. Rolling back
    /// the application does not restore it.
    ///
    /// Update your application code to no longer call the function, then drop it in a later
    /// migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop function calculate_total(int);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// drop procedure archive_orders();
    /// ```
    ///
    pub BanDropFunction {
        version: "next",
        name: "banDropFunction",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-function")],
    }
}

impl LinterRule for BanDropFunction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        let kind = match stmt.remove_type() {
            ObjectType::ObjectFunction => "a function",
            ObjectType::ObjectProcedure => "a procedure",
            ObjectType::ObjectRoutine => "a routine",
            ObjectType::ObjectAggregate => "an aggregate",
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Dropping "{kind}" may break existing clients."
                },
            )
            .detail(
                None,
                "Clients that still call it fail, and rolling back the application does not restore it.",
            )
            .note("Update your application code to no longer call it, then drop it in a later migration."),
        ]
    }
}
