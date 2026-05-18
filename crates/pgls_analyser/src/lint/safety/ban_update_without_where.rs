use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An `UPDATE` statement without a `WHERE` clause will modify all rows in the table.
    ///
    /// This is almost always unintentional in a migration context and can cause data corruption.
    /// If you truly need to update all rows, add a `WHERE true` to signal intent.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// update my_table set col = 'value';
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// update my_table set col = 'value' where id = 1;
    /// ```
    ///
    pub BanUpdateWithoutWhere {
        version: "next",
        name: "banUpdateWithoutWhere",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("update-in-migration")],
    }
}

impl LinterRule for BanUpdateWithoutWhere {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::UpdateStmt(stmt) = &ctx.stmt()
            && stmt.where_clause.is_none()
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "An "<Emphasis>"UPDATE"</Emphasis>" without a "<Emphasis>"WHERE"</Emphasis>" clause will modify all rows in the table."
                    },
                )
                .detail(
                    None,
                    "Add a WHERE clause to limit which rows are updated.",
                ),
            );
        }

        diagnostics
    }
}
