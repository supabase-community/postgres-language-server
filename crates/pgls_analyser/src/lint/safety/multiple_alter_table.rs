use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Multiple ALTER TABLE statements on the same table should be combined into a single statement.
    ///
    /// When you run multiple ALTER TABLE statements on the same table, Postgres must scan and potentially
    /// rewrite the table multiple times. Each ALTER TABLE command requires acquiring locks and performing
    /// table operations that can be expensive, especially on large tables.
    ///
    /// Combining multiple ALTER TABLE operations into a single statement with comma-separated actions
    /// allows Postgres to scan and modify the table only once, improving performance and reducing
    /// the time locks are held.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
    /// ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE authors
    ///   ALTER COLUMN name SET NOT NULL,
    ///   ALTER COLUMN email SET NOT NULL;
    /// ```
    ///
    pub MultipleAlterTable {
        version: "0.17.0",
        name: "multipleAlterTable",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Eugene("W12")],
    }
}

impl LinterRule for MultipleAlterTable {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check if current statement is ALTER TABLE
        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt()
            && let Some(relation) = &stmt.relation
        {
            let session = ctx.session();
            if session.has_altered_table(&relation.schemaname, &relation.relname) {
                let schema = session.object_schema(&relation.schemaname);
                let table = &relation.relname;

                diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Multiple "<Emphasis>"ALTER TABLE"</Emphasis>" statements found for table "<Emphasis>{schema}"."{{table}}</Emphasis>"."
                            },
                        )
                        .detail(
                            None,
                            "Multiple ALTER TABLE statements on the same table require scanning and potentially rewriting the table multiple times.",
                        )
                        .note("Combine the ALTER TABLE statements into a single statement with comma-separated actions to scan the table only once."),
                    );
            }
        }

        diagnostics
    }
}
