use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Setting a column NOT NULL blocks reads while the table is scanned.
    ///
    /// Setting NOT NULL on an existing column scans the table under an ACCESS EXCLUSIVE lock,
    /// blocking reads and writes. On PostgreSQL 12+, a validated CHECK (column IS NOT NULL)
    /// constraint allows PostgreSQL to skip this scan.
    ///
    /// Instead of using SET NOT NULL, consider using a CHECK constraint with NOT VALID, then
    /// validating it in a separate transaction. This allows reads and writes to continue.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- First add a CHECK constraint as NOT VALID
    /// ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null CHECK (foo IS NOT NULL) NOT VALID;
    /// -- Then validate it in a separate transaction
    /// ALTER TABLE "core_recipe" VALIDATE CONSTRAINT foo_not_null;
    /// ```
    ///
    pub AddingNotNullField {
        version: "0.15.0",
        name: "addingNotNullField",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("adding-not-null-field")],
    }
}

impl LinterRule for AddingNotNullField {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtSetNotNull
                {
                    diagnostics.push(LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Setting a column NOT NULL blocks reads while the table is scanned."
                            },
                        ).detail(None, "This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.")
                        .note("On PostgreSQL 12+, a validated CHECK (column IS NOT NULL) constraint lets PostgreSQL skip the scan."));
                }
            }
        }

        diagnostics
    }
}
