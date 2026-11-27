use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_diagnostics::Severity;
use pgls_resolve::FindingKind;

declare_lint_rule! {
    /// A `SELECT ... INTO` that Postgres rejects.
    ///
    /// In a set operation like `UNION`, only the first select may have an `INTO`.
    ///
    /// Postgres raises `42601 syntax_error`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select 1 as id union select 2 into ids;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1 as id into ids union select 2;
    /// ```
    ///
    pub InvalidSelectInto {
        version: "next",
        name: "invalidSelectInto",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for InvalidSelectInto {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        ctx.findings()
            .filter(|finding| matches!(finding.kind, FindingKind::IntoNotOnFirstSelect))
            .map(|finding| {
                LinterDiagnostic::new(
                    rule_category!(),
                    finding.span,
                    "INTO is only allowed on first SELECT of UNION/INTERSECT/EXCEPT.",
                )
            })
            .collect()
    }
}
