use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Renaming tables may break existing queries and application code.
    ///
    /// Renaming a table that is being referenced by existing applications, views, functions, or foreign keys
    /// can cause unexpected downtime. Consider creating a view with the old table name pointing to the new table,
    /// or carefully coordinate the rename with application deployments.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users RENAME TO app_users;
    /// ```
    ///
    pub RenamingTable {
        version: "next",
        name: "renamingTable",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("renaming-table")],
    }
}

impl Rule for RenamingTable {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::RenameStmt(stmt) = &ctx.stmt() {
            if stmt.rename_type() == pgls_query::protobuf::ObjectType::ObjectTable {
                diagnostics.push(RuleDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Renaming a table may break existing clients."
                    },
                ).detail(None, "Consider creating a view with the old table name instead, or coordinate the rename carefully with application deployments."));
            }
        }

        diagnostics
    }
}
