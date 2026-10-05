use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An unqualified column name matches columns of more than one relation in scope.
    ///
    /// Qualify the column with the name or alias of the relation it belongs to.
    ///
    /// Postgres raises `42702 ambiguous_column` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8, name text);
    /// create table posts (id int8, user_id int8);
    /// select id from users join posts on posts.user_id = users.id;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8, name text);
    /// create table posts (id int8, user_id int8);
    /// select users.id from users join posts on posts.user_id = users.id;
    /// ```
    ///
    pub AmbiguousColumn {
        version: "0.27.0",
        name: "ambiguousColumn",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for AmbiguousColumn {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::AmbiguousColumn { column, candidates } => {
                    let candidates = candidates.join(", ");
                    Some(
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Column reference "<Emphasis>{column}</Emphasis>" is ambiguous." },
                        )
                        .note(markup! { "It matches columns of "{candidates}"." }),
                    )
                }
                _ => None,
            })
            .collect()
    }
}
