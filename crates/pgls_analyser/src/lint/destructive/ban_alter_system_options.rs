use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Changing the server configuration may change behaviour for existing clients.
    ///
    /// `ALTER SYSTEM` writes settings to `postgresql.auto.conf`, which apply to every database
    /// and session of the server once the configuration is reloaded.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter system set timezone = 'UTC';
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter system reset all;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// set timezone = 'UTC';
    /// ```
    ///
    pub BanAlterSystemOptions {
        version: "next",
        name: "banAlterSystemOptions",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-system-options")],
    }
}

impl LinterRule for BanAlterSystemOptions {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterSystemStmt(_) = ctx.stmt() else {
            return vec![];
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Changing the server configuration may change behaviour for existing clients."
                },
            )
            .detail(
                None,
                "The settings apply to every database and session of the server once the configuration is reloaded.",
            )
            .note("Change the setting for a database or role instead, or check that every client of the server works with it."),
        ]
    }
}
