use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A column does not exist on the relation or record it is taken from.
    ///
    /// Columns are resolved against the relations in scope: tables, views, subqueries, CTEs,
    /// functions in `FROM`, and the parameters of SQL functions.
    ///
    /// Postgres raises `42703 undefined_column` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8, name text);
    /// select email from users;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8, name text);
    /// select name from users;
    /// ```
    ///
    pub UnknownColumn {
        version: "next",
        name: "unknownColumn",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for UnknownColumn {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::UnknownColumn { relation, column } => {
                    Some(match relation {
                        Some(relation) => LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Column "<Emphasis>{column}</Emphasis>" does not exist on "<Emphasis>{relation}</Emphasis>"." },
                        ),
                        None => LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Column "<Emphasis>{column}</Emphasis>" does not exist." },
                        ),
                    })
                }
                _ => None,
            })
            .collect()
    }
}
