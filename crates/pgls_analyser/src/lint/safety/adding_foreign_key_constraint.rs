use crate::utils::{alter_table_cmds, cmd_def};
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
    /// A `NOT ENFORCED` foreign key (Postgres 18) isn't checked, so adding one doesn't scan the table.
    /// Making it `ENFORCED` later with `ALTER CONSTRAINT` checks all rows while holding an ACCESS EXCLUSIVE
    /// lock, and can't be split into NOT VALID and VALIDATE.
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
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "email" ALTER CONSTRAINT "fk_user" ENFORCED;
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
        let mut diagnostics = Vec::new();

        for cmd in alter_table_cmds(ctx.stmt()) {
            match cmd.subtype() {
                pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                    if let Some(pgls_query::NodeEnum::Constraint(constraint)) = cmd_def(cmd)
                        && let Some(diagnostic) = check_foreign_key_constraint(constraint, false)
                    {
                        diagnostics.push(diagnostic);
                    }
                }
                pgls_query::protobuf::AlterTableType::AtAddColumn => {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = cmd_def(cmd) {
                        // check constraints on the column
                        let constraints: Vec<_> = col_def
                            .constraints
                            .iter()
                            .filter_map(|constraint| match &constraint.node {
                                Some(pgls_query::NodeEnum::Constraint(constraint)) => {
                                    Some(constraint.as_ref())
                                }
                                _ => None,
                            })
                            .collect();
                        for (index, constr) in constraints.iter().enumerate() {
                            let following = constraints.get(index + 1..).unwrap_or_default();
                            if !is_not_enforced(following)
                                && let Some(diagnostic) = check_foreign_key_constraint(constr, true)
                            {
                                diagnostics.push(diagnostic);
                            }
                        }
                    }
                }
                pgls_query::protobuf::AlterTableType::AtAlterConstraint => {
                    if let Some(pgls_query::NodeEnum::AtalterConstraint(alter)) = cmd_def(cmd)
                        && alter.alter_enforceability
                        && alter.is_enforced
                    {
                        diagnostics.push(enforce_diagnostic(&alter.conname));
                    }
                }
                _ => {}
            }
        }

        diagnostics
    }
}

/// Whether the attributes that follow a column constraint, up to the next constraint, include
/// `NOT ENFORCED`. Like in [`transformConstraintAttrs`], a `NOT ENFORCED` foreign key isn't
/// validated.
///
/// [`transformConstraintAttrs`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L3909
fn is_not_enforced(following: &[&pgls_query::protobuf::Constraint]) -> bool {
    use pgls_query::protobuf::ConstrType;
    following
        .iter()
        .take_while(|constraint| {
            matches!(
                constraint.contype(),
                ConstrType::ConstrAttrDeferrable
                    | ConstrType::ConstrAttrNotDeferrable
                    | ConstrType::ConstrAttrDeferred
                    | ConstrType::ConstrAttrImmediate
                    | ConstrType::ConstrAttrEnforced
                    | ConstrType::ConstrAttrNotEnforced
            )
        })
        .any(|constraint| constraint.contype() == ConstrType::ConstrAttrNotEnforced)
}

/// `ALTER CONSTRAINT ... ENFORCED` checks all rows of a foreign key that was `NOT ENFORCED`
/// ([`ATExecAlterConstrEnforceability`]).
///
/// [`ATExecAlterConstrEnforceability`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L12428
fn enforce_diagnostic(name: &str) -> LinterDiagnostic {
    LinterDiagnostic::new(
        rule_category!(),
        None,
        markup! { "Enforcing foreign key constraint "<Emphasis>{name}</Emphasis>" checks all rows of the table." },
    )
    .detail(
        None,
        "This holds an ACCESS EXCLUSIVE lock on the table while Postgres verifies the constraint.",
    )
    .note("Drop the constraint and add it again as NOT VALID, then VALIDATE it in a separate transaction.")
}

fn check_foreign_key_constraint(
    constraint: &pgls_query::protobuf::Constraint,
    is_column_constraint: bool,
) -> Option<LinterDiagnostic> {
    // Only check foreign key constraints
    if constraint.contype() != pgls_query::protobuf::ConstrType::ConstrForeign {
        return None;
    }

    // NOT VALID constraints are safe, and so are NOT ENFORCED ones, which are never validated
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
