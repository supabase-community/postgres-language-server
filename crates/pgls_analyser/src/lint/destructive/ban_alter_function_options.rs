use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Changing the options of a function, procedure or routine may change behaviour for existing
    /// clients.
    ///
    /// Options such as `SECURITY DEFINER`, `SET search_path`, the volatility or `STRICT` change
    /// with which privileges the function runs, which objects it finds, and what it returns.
    ///
    /// Renaming the function, changing its owner or its schema are not reported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter function add_user(text) security definer;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter procedure archive_orders() set search_path to private;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter function add_user(text) rename to create_user;
    /// ```
    ///
    pub BanAlterFunctionOptions {
        version: "next",
        name: "banAlterFunctionOptions",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-function-options")],
    }
}

impl LinterRule for BanAlterFunctionOptions {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterFunctionStmt(stmt) = ctx.stmt() else {
            return vec![];
        };
        let object = match stmt.objtype() {
            ObjectType::ObjectProcedure => "procedure",
            ObjectType::ObjectRoutine => "routine",
            _ => "function",
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Changing the options of a "{object}" may change behaviour for existing clients."
                },
            )
            .detail(
                None,
                "The options decide with which privileges and settings it runs, and how Postgres calls it.",
            )
            .note("Check that existing clients work with the new options before changing them."),
        ]
    }
}
