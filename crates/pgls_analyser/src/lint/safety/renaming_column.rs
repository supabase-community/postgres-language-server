use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Renaming columns may break existing queries and application code.
    ///
    /// Renaming a column that is being used by an existing application or query can cause unexpected downtime.
    /// Consider creating a new column instead and migrating the data, then dropping the old column after ensuring
    /// no dependencies exist.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users RENAME COLUMN email TO email_address;
    /// ```
    ///
    pub RenamingColumn {
        version: "next",
        name: "renamingColumn",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("renaming-column")],
    }
}

impl LinterRule for RenamingColumn {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::RenameStmt(stmt) = &ctx.stmt() {
            if stmt.rename_type() == pgls_query::protobuf::ObjectType::ObjectColumn {
                diagnostics.push(LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Renaming a column may break existing clients."
                    },
                ).detail(None, "Consider creating a new column with the desired name and migrating data instead."));
            }
        }

        diagnostics
    }
}
