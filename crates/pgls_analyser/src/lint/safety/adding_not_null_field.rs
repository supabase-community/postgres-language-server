use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Setting a column NOT NULL blocks reads while the table is scanned.
    ///
    /// Setting NOT NULL on an existing column scans the table under an ACCESS EXCLUSIVE lock,
    /// blocking reads and writes. This applies to `ALTER COLUMN ... SET NOT NULL` and, on
    /// Postgres 18, to adding a NOT NULL constraint with `ADD CONSTRAINT ... NOT NULL` without
    /// `NOT VALID`.
    ///
    /// On Postgres 18, add the NOT NULL constraint as NOT VALID, then validate it in a separate
    /// transaction. Validating only takes a SHARE UPDATE EXCLUSIVE lock, so reads and writes can
    /// continue.
    ///
    /// Before Postgres 18, a validated CHECK (column IS NOT NULL) constraint lets SET NOT NULL
    /// skip the scan (Postgres 12+). Add the CHECK constraint as NOT VALID and validate it in a
    /// separate transaction first.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null NOT NULL "foo";
    /// ```
    ///
    /// ### Valid
    ///
    /// On Postgres 18:
    ///
    /// ```sql
    /// -- First add the NOT NULL constraint as NOT VALID
    /// ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null NOT NULL "foo" NOT VALID;
    /// -- Then validate it in a separate transaction
    /// ALTER TABLE "core_recipe" VALIDATE CONSTRAINT foo_not_null;
    /// ```
    ///
    /// Before Postgres 18:
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

/// The note on how to add NOT NULL without the scan.
const NOT_VALID_NOTE: &str = "Add the constraint with ADD CONSTRAINT ... NOT NULL ... NOT VALID, then VALIDATE it in a separate transaction.";
const CHECK_NOTE: &str = "On PostgreSQL 12+, a validated CHECK (column IS NOT NULL) constraint lets PostgreSQL skip the scan.";

impl LinterRule for AddingNotNullField {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        // `NOT NULL ... NOT VALID` is new in Postgres 18.
        let supports_not_valid = ctx
            .snapshot()
            .and_then(|snapshot| snapshot.version.major_version)
            .is_some_and(|major| major >= 18);

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node else {
                    continue;
                };
                let note = match cmd.subtype() {
                    pgls_query::protobuf::AlterTableType::AtSetNotNull => {
                        if supports_not_valid {
                            NOT_VALID_NOTE
                        } else {
                            CHECK_NOTE
                        }
                    }
                    // `ADD CONSTRAINT ... NOT NULL column`, which only parses on Postgres 18.
                    pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                        match cmd.def.as_ref().and_then(|d| d.node.as_ref()) {
                            Some(pgls_query::NodeEnum::Constraint(constraint))
                                if constraint.contype()
                                    == pgls_query::protobuf::ConstrType::ConstrNotnull
                                    && !constraint.skip_validation =>
                            {
                                NOT_VALID_NOTE
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                };
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Setting a column NOT NULL blocks reads while the table is scanned."
                        },
                    )
                    .detail(None, "This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.")
                    .note(note),
                );
            }
        }

        diagnostics
    }
}
