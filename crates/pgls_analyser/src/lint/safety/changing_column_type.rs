use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Changing a column type may break existing clients.
    ///
    /// Changing a column's data type requires an exclusive lock on the table while the entire table is rewritten.
    /// This can take a long time for large tables and will block reads and writes.
    ///
    /// Instead of changing the type directly, consider creating a new column with the desired type,
    /// migrating the data, and then dropping the old column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text USING "edits"::text;
    /// ```
    ///
    pub ChangingColumnType {
        version: "next",
        name: "changingColumnType",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("changing-column-type")],
    }
}

impl Rule for ChangingColumnType {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAlterColumnType {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Changing a column type requires a table rewrite and blocks reads and writes."
                            }
                        ).detail(None, "Consider creating a new column with the desired type, migrating data, and then dropping the old column."));
                    }
                }
            }
        }

        diagnostics
    }
}
