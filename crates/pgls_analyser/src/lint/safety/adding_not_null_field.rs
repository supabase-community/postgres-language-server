use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Setting a column NOT NULL blocks reads while the table is scanned.
    ///
    /// In PostgreSQL versions before 11, adding a NOT NULL constraint to an existing column requires
    /// a full table scan to verify that all existing rows satisfy the constraint. This operation
    /// takes an ACCESS EXCLUSIVE lock, blocking all reads and writes.
    ///
    /// In PostgreSQL 11+, this operation is much faster as it can skip the full table scan for
    /// newly added columns with default values.
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
        version: "next",
        name: "addingNotNullField",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("adding-not-null-field")],
    }
}

impl Rule for AddingNotNullField {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // In Postgres 11+, this is less of a concern
        if ctx
            .schema_cache()
            .is_some_and(|sc| sc.version.major_version.is_some_and(|v| v >= 11))
        {
            return diagnostics;
        }

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtSetNotNull {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Setting a column NOT NULL blocks reads while the table is scanned."
                            },
                        ).detail(None, "This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.")
                        .note("Use a CHECK constraint with NOT VALID instead, then validate it in a separate transaction."));
                    }
                }
            }
        }

        diagnostics
    }
}
