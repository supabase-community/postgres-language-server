use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::typing::format_type;
use pgls_diagnostics::Severity;
use pgls_query::QualifiedName;
use pgls_resolve::{FindingKind, MatchFailure};

declare_lint_rule! {
    /// A function name and argument count exist, but its argument types do not select exactly
    /// one overload, or `CALL` selects a function that isn't a procedure.
    ///
    /// Postgres raises `42883 undefined_function` when no function matches,
    /// `42725 ambiguous_function` when the call is ambiguous, and `42809 wrong_object_type` when
    /// `CALL` selects a function.
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
        let mut diagnostics = Vec::new();
        for finding in ctx.findings() {
            let (schema, name, args, failure) = match &finding.kind {
                FindingKind::FunctionArgumentMismatch {
                    schema,
                    name,
                    args,
                    failure,
                    procedure,
                } => (schema, name, args, Some((*failure, *procedure))),
                FindingKind::NotAProcedure { schema, name, args } => (schema, name, args, None),
                _ => continue,
            };
            let catalog = ctx.catalog();
            let Some(args) = args
                .iter()
                .map(|arg| format_type(catalog, arg))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let name = QualifiedName {
                catalog: None,
                schema: schema.as_deref(),
                name,
            }
            .to_string();
            let call = format!("{name}({})", args.join(", "));
            let (message, note) = match failure {
                Some((MatchFailure::NoMatch, false)) => (
                    format!("Function {call} does not exist."),
                    "No function matches the given name and argument types. You might need to add explicit type casts.",
                ),
                Some((MatchFailure::Ambiguous, false)) => (
                    format!("Function {call} is not unique."),
                    "Could not choose a best candidate function. You might need to add explicit type casts.",
                ),
                Some((MatchFailure::NoMatch, true)) => (
                    format!("Procedure {call} does not exist."),
                    "No procedure matches the given name and argument types. You might need to add explicit type casts.",
                ),
                Some((MatchFailure::Ambiguous, true)) => (
                    format!("Procedure {call} is not unique."),
                    "Could not choose a best candidate procedure. You might need to add explicit type casts.",
                ),
                None => (
                    format!("{call} is not a procedure."),
                    "To call a function, use SELECT.",
                ),
            };
            diagnostics
                .push(LinterDiagnostic::new(rule_category!(), finding.span, message).note(note));
        }
        diagnostics
    }
}
