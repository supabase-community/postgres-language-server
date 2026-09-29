use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A column is qualified with a name that is not in the `FROM` clause.
    ///
    /// The qualifier must be the name or alias of a relation in scope. After aliasing a table, its
    /// original name can't be used anymore.
    ///
    /// Postgres raises `42P01 undefined_table` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8);
    /// select users.id from users u;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8);
    /// select u.id from users u;
    /// ```
    ///
    pub MissingFromClauseEntry {
        version: "next",
        name: "missingFromClauseEntry",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for MissingFromClauseEntry {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::MissingFromClauseEntry { name } => Some(LinterDiagnostic::new(
                    rule_category!(),
                    finding.span,
                    markup! { "Missing FROM-clause entry for "<Emphasis>{name}</Emphasis>"." },
                )),
                _ => None,
            })
            .collect()
    }
}
