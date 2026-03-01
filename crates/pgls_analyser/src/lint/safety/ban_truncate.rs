use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Truncating a table removes all rows and can cause data loss in production.
    ///
    /// `TRUNCATE` is a fast, non-transactional (in terms of row-level locking) way to remove
    /// all data from a table. It acquires an `ACCESS EXCLUSIVE` lock and cannot be safely
    /// rolled back in all scenarios. In a migration context, this is almost always a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// truncate my_table;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// delete from my_table where expired_at < now();
    /// ```
    ///
    pub BanTruncate {
        version: "next",
        name: "banTruncate",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("truncate")],
    }
}

impl LinterRule for BanTruncate {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::TruncateStmt(_) = &ctx.stmt() {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Truncating a table removes all rows and can cause data loss."
                    },
                )
                .detail(
                    None,
                    "Use DELETE with a WHERE clause instead, or ensure this is intentional and not part of a migration.",
                ),
            );
        }

        diagnostics
    }
}
