use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{FindingKind, MatchFailure};
use pgls_catalog::typing::format_type;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An operator exists by name but cannot be resolved for the operand types, or has
    /// multiple equally suitable candidates.
    ///
    /// The rule needs a database connection to load the operator and type catalog.
    /// Postgres reports SQLSTATE `42883` when no operator matches and `42725` when an
    /// operator is ambiguous.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select 1 + timestamp '2020-01-01';
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1 + 2;
    /// ```
    ///
    pub OperatorTypeMismatch {
        version: "next",
        name: "operatorTypeMismatch",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for OperatorTypeMismatch {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };
        let mut diagnostics = Vec::new();
        for finding in &resolution.findings {
            let FindingKind::OperatorMismatch {
                operator,
                left,
                right,
                failure,
            } = &finding.kind
            else {
                continue;
            };
            let catalog = ctx.catalog();
            let Some(right) = format_type(catalog, right) else {
                continue;
            };
            let left = match left {
                Some(left) => match format_type(catalog, left) {
                    Some(left) => Some(left),
                    None => continue,
                },
                None => None,
            };
            let operands = match left {
                Some(left) => format!("{left} {operator} {right}"),
                None => format!("{operator} {right}"),
            };
            let (message, note) = match failure {
                MatchFailure::NoMatch => (
                    format!("Operator does not exist: {operands}"),
                    "No operator matches the given name and argument types. You might need to add explicit type casts.",
                ),
                MatchFailure::Ambiguous => (
                    format!("Operator is not unique: {operands}"),
                    "Could not choose a best candidate operator. You might need to add explicit type casts.",
                ),
            };
            diagnostics
                .push(LinterDiagnostic::new(rule_category!(), finding.span, message).note(note));
        }
        diagnostics
    }
}
