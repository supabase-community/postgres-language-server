use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ConstrType;

declare_lint_rule! {
    /// Domains with constraints have poor support for online migrations.
    ///
    /// Any later change to the constraint of a domain revalidates every column that uses the
    /// domain, and coercing a column to a domain with constraints rewrites the table. Postgres
    /// can't reason about arbitrary domain constraints the way it can about simple column types.
    ///
    /// Put the constraint on the columns directly instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create domain positive_int as int check (value > 0);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create domain name_text as text collate "C";
    /// ```
    ///
    pub BanCreateDomainWithConstraint {
        version: "next",
        name: "banCreateDomainWithConstraint",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-create-domain-with-constraint")],
    }
}

impl LinterRule for BanCreateDomainWithConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // `DEFAULT` and `NULL` are constraint nodes too, but don't restrict the values.
        if let pgls_query::NodeEnum::CreateDomainStmt(stmt) = ctx.stmt()
            && stmt.constraints.iter().any(|constraint| {
                matches!(
                    &constraint.node,
                    Some(pgls_query::NodeEnum::Constraint(constraint))
                        if !matches!(constraint.contype(), ConstrType::ConstrDefault | ConstrType::ConstrNull)
                )
            })
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Domains with constraints have poor support for online migrations."
                    },
                )
                .detail(
                    None,
                    "Changing the constraint later revalidates every column that uses the domain.",
                )
                .note("Use table and column constraints instead."),
            ];
        }

        vec![]
    }
}
