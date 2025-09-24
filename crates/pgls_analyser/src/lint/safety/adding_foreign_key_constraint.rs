use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes.
    ///
    /// Adding a foreign key constraint to an existing table can cause downtime by locking both tables while
    /// verifying the constraint. PostgreSQL needs to check that all existing values in the referencing
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
        version: "next",
        name: "addingForeignKeyConstraint",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("adding-foreign-key-constraint")],
    }
}

impl Rule for AddingForeignKeyConstraint {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    match cmd.subtype() {
                        pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                            if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                                cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                if let Some(diagnostic) =
                                    check_foreign_key_constraint(constraint, false)
                                {
                                    diagnostics.push(diagnostic);
                                }
                            }
                        }
                        pgls_query::protobuf::AlterTableType::AtAddColumn => {
                            if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                // check constraints on the column
                                for constraint in &col_def.constraints {
                                    if let Some(pgls_query::NodeEnum::Constraint(constr)) =
                                        &constraint.node
                                    {
                                        if let Some(diagnostic) =
                                            check_foreign_key_constraint(constr, true)
                                        {
                                            diagnostics.push(diagnostic);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        diagnostics
    }
}

fn check_foreign_key_constraint(
    constraint: &pgls_query::protobuf::Constraint,
    is_column_constraint: bool,
) -> Option<RuleDiagnostic> {
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
            "This will block writes to both the referencing and referenced tables while PostgreSQL verifies the constraint.",
            "Add the constraint as NOT VALID first, then VALIDATE it in a separate transaction.",
        )
    };

    Some(
        RuleDiagnostic::new(rule_category!(), None, markup! { {message} })
            .detail(None, detail)
            .note(note),
    )
}
