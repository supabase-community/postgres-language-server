use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An `INSERT` has a different number of target columns than values.
    ///
    /// With an explicit column list, every row of `VALUES` (or the `SELECT` list) must have
    /// exactly one value per column. Without one, there can't be more values than the table has
    /// columns.
    ///
    /// Postgres raises `42601 syntax_error` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8, name text);
    /// insert into users (id, name) values (1);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8, name text);
    /// insert into users (id, name) values (1, 'Ada');
    /// ```
    ///
    pub InsertColumnMismatch {
        version: "0.27.0",
        name: "insertColumnMismatch",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for InsertColumnMismatch {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::InsertColumnMismatch { expected, found } => {
                    let message = if found > expected {
                        "INSERT has more expressions than target columns."
                    } else {
                        "INSERT has more target columns than expressions."
                    };
                    Some(
                        LinterDiagnostic::new(rule_category!(), finding.span, message)
                            .note(markup! { "Expected "{expected}" values, found "{found}"." }),
                    )
                }
                _ => None,
            })
            .collect()
    }
}
