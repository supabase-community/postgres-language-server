use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{FindingKind, MatchFailure};
use pgls_catalog::typing::format_type;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A function name and argument count exist, but its argument types do not select exactly
    /// one overload.
    ///
    /// Postgres raises `42883 undefined_function` when no function matches, and
    /// `42725 ambiguous_function` when the call is ambiguous.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create function example_fn(value integer) returns integer language sql as 'select value';
    /// select example_fn(timestamp '2020-01-01');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create function example_fn(value integer) returns integer language sql as 'select value';
    /// select example_fn(1);
    /// ```
    ///
    pub FunctionArgumentMismatch {
        version: "0.27.0",
        name: "functionArgumentMismatch",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for FunctionArgumentMismatch {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };
        let mut diagnostics = Vec::new();
        for finding in &resolution.findings {
            let FindingKind::FunctionArgumentMismatch {
                schema,
                name,
                args,
                failure,
            } = &finding.kind
            else {
                continue;
            };
            let catalog = ctx.catalog();
            let Some(args) = args
                .iter()
                .map(|arg| format_type(catalog, arg))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let name = match schema {
                Some(schema) => format!("{schema}.{name}"),
                None => name.clone(),
            };
            let call = format!("{name}({})", args.join(", "));
            let (message, note) = match failure {
                MatchFailure::NoMatch => (
                    format!("Function {call} does not exist."),
                    "No function matches the given name and argument types. You might need to add explicit type casts.",
                ),
                MatchFailure::Ambiguous => (
                    format!("Function {call} is not unique."),
                    "Could not choose a best candidate function. You might need to add explicit type casts.",
                ),
            };
            diagnostics
                .push(LinterDiagnostic::new(rule_category!(), finding.span, message).note(note));
        }
        diagnostics
    }
}
