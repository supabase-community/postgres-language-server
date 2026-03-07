use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A `DELETE` statement without a `WHERE` clause will remove all rows from the table.
    ///
    /// This is almost always unintentional in a migration or application context.
    /// If you truly need to remove all rows, use `TRUNCATE` explicitly (and acknowledge
    /// its implications), or add a `WHERE true` to signal intent.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// delete from my_table;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// delete from my_table where expired_at < now();
    /// ```
    ///
    pub BanDeleteWithoutWhere {
        version: "next",
        name: "banDeleteWithoutWhere",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("delete-without-where")],
    }
}

impl LinterRule for BanDeleteWithoutWhere {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::DeleteStmt(stmt) = &ctx.stmt()
            && stmt.where_clause.is_none()
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "A "<Emphasis>"DELETE"</Emphasis>" without a "<Emphasis>"WHERE"</Emphasis>" clause will remove all rows from the table."
                    },
                )
                .detail(
                    None,
                    "Add a WHERE clause to limit which rows are deleted.",
                ),
            );
        }

        diagnostics
    }
}
