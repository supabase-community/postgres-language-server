use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a constraint to a domain has poor support for online migrations.
    ///
    /// `ALTER DOMAIN ... ADD CONSTRAINT` validates every column that uses the domain, across all
    /// tables, while holding locks on them. Postgres can't reason about arbitrary domain
    /// constraints the way it can about simple column types.
    ///
    /// Put the constraint on the columns directly instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter domain positive_int add constraint positive_int_check check (value > 0);
    /// ```
    ///
    pub BanAlterDomainWithAddConstraint {
        version: "next",
        name: "banAlterDomainWithAddConstraint",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-domain-with-add-constraint")],
    }
}

impl LinterRule for BanAlterDomainWithAddConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // 'C' is `ADD CONSTRAINT`.
        if let pgls_query::NodeEnum::AlterDomainStmt(stmt) = ctx.stmt()
            && stmt.subtype == "C"
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Adding a constraint to a domain has poor support for online migrations."
                    },
                )
                .detail(
                    None,
                    "Postgres validates every column that uses the domain, across all tables.",
                )
                .note("Use table and column constraints instead."),
            ];
        }

        vec![]
    }
}
