use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_catalog::typing::format_type;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An explicit cast is not permitted between the source and target types. The rule needs
    /// a database connection to load the type and cast catalog.
    ///
    /// Postgres reports SQLSTATE `42846` (`cannot_coerce`).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select timestamp '2020-01-01'::integer;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1::text;
    /// ```
    ///
    pub InvalidCast {
        version: "next",
        name: "invalidCast",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for InvalidCast {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };
        resolution
            .findings
            .iter()
            .filter_map(|finding| {
                let FindingKind::InvalidCast { from, to } = &finding.kind else {
                    return None;
                };
                let catalog = ctx.catalog();
                let (Some(from), Some(to)) = (format_type(catalog, from), format_type(catalog, to))
                else {
                    return None;
                };
                Some(LinterDiagnostic::new(
                    rule_category!(),
                    finding.span,
                    format!("Cannot cast type {from} to {to}."),
                ))
            })
            .collect()
    }
}
