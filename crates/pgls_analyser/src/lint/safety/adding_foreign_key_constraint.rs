use crate::utils::{ConstraintSite, added_constraints, altered_relation};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes.
    ///
    /// Adding a foreign key constraint to an existing table can cause downtime by locking both tables while
    /// verifying the constraint. Postgres needs to check that all existing values in the referencing
    /// column exist in the referenced table.
    ///
    /// Instead, add the constraint as NOT VALID in one transaction, then VALIDATE it in another transaction.
    /// This approach only takes a SHARE UPDATE EXCLUSIVE lock when validating, allowing concurrent writes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "emails" ADD COLUMN "user_id" INT REFERENCES "user" ("id");
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- First add the constraint as NOT VALID
    /// ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id") NOT VALID;
    /// -- Then validate it in a separate transaction
    /// ALTER TABLE "email" VALIDATE CONSTRAINT "fk_user";
    /// ```
    ///
    pub AddingForeignKeyConstraint {
        version: "0.15.0",
        name: "addingForeignKeyConstraint",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("adding-foreign-key-constraint")],
    }
}

impl LinterRule for AddingForeignKeyConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // Other transactions can't see a table created earlier in the transaction, so the scan
        // blocks no one. The referenced table is locked until the end of the transaction, like
        // by `CREATE TABLE ... REFERENCES`, and NOT VALID wouldn't change that.
        if altered_relation(ctx.stmt()).is_some_and(|relation| ctx.is_new_relation(relation)) {
            return vec![];
        }
        added_constraints(ctx.stmt())
            .filter_map(|(constraint, site)| {
                check_foreign_key_constraint(constraint, site == ConstraintSite::Column)
            })
            .collect()
    }
}

fn check_foreign_key_constraint(
    constraint: &pgls_query::protobuf::Constraint,
    is_column_constraint: bool,
) -> Option<LinterDiagnostic> {
    // Only check foreign key constraints
    if constraint.contype() != pgls_query::protobuf::ConstrType::ConstrForeign {
        return None;
    }

    // NOT VALID constraints are safe
    if constraint.skip_validation {
        return None;
    }

    let (message, detail, note) = if is_column_constraint {
        (
            "Adding a column with a foreign key constraint requires a table scan and locks.",
            "Using REFERENCES when adding a column will block writes while verifying the constraint.",
            "Add the column without the constraint first, then add the constraint as NOT VALID and VALIDATE it separately.",
        )
    } else {
        (
            "Adding a foreign key constraint requires a table scan and locks on both tables.",
            "This will block writes to both the referencing and referenced tables while Postgres verifies the constraint.",
            "Add the constraint as NOT VALID first, then VALIDATE it in a separate transaction.",
        )
    };

    Some(
        LinterDiagnostic::new(rule_category!(), None, markup! { {message} })
            .detail(None, detail)
            .note(note),
    )
}
