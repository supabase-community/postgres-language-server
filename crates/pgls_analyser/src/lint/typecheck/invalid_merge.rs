use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{FindingKind, MergeError};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A `MERGE`, or `merge_action()`, that Postgres rejects.
    ///
    /// Postgres rejects a `MERGE` with `WITH RECURSIVE`, a `WHEN` clause after an unconditional
    /// `WHEN` clause of the same kind, which can never run, and a target and source with the same
    /// name. `merge_action()` is only allowed in the `RETURNING` list of a `MERGE`.
    ///
    /// Postgres raises `42712 duplicate_alias` for the same name, and `42601 syntax_error` for the
    /// others.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8, name text);
    /// create table imports (id int8, name text);
    /// merge into users using imports on imports.id = users.id
    /// when matched then delete
    /// when matched and imports.name is null then do nothing;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8, name text);
    /// create table imports (id int8, name text);
    /// merge into users using imports on imports.id = users.id
    /// when matched and imports.name is null then do nothing
    /// when matched then delete;
    /// ```
    ///
    pub InvalidMerge {
        version: "next",
        name: "invalidMerge",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for InvalidMerge {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        ctx.findings()
            .filter_map(|finding| {
                let FindingKind::InvalidMerge(error) = &finding.kind else {
                    return None;
                };
                let diagnostic = match error {
                    MergeError::RecursiveWith => LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        "WITH RECURSIVE is not supported for MERGE statement.",
                    ),
                    MergeError::UnreachableWhen => LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        "unreachable WHEN clause specified after unconditional WHEN clause.",
                    )
                    .note(markup! {
                        "A WHEN clause without a condition matches every row of its kind first."
                    }),
                    MergeError::DuplicateName { name } => LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        markup! { "name \""{name}"\" specified more than once." },
                    )
                    .note("The name is used both as MERGE target table and data source."),
                    MergeError::MergeActionOutsideReturning => LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        "MERGE_ACTION() can only be used in the RETURNING list of a MERGE command.",
                    ),
                };
                Some(diagnostic)
            })
            .collect()
    }
}
