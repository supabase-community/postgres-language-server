use pgt_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgt_console::markup;
use pgt_diagnostics::Severity;

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

        if let pgt_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgt_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    match cmd.subtype() {
                        pgt_query::protobuf::AlterTableType::AtAddConstraint => {
                            if let Some(def) = cmd.def.as_ref().and_then(|d| d.node.as_ref()) {
                                if let pgt_query::NodeEnum::Constraint(constraint) = def {
                                    // check if it's a foreign key constraint
                                    if constraint.contype()
                                        == pgt_query::protobuf::ConstrType::ConstrForeign
                                    {
                                        // it is okay if NOT VALID is specified (skip_validation = true)
                                        if !constraint.skip_validation {
                                            diagnostics.push(RuleDiagnostic::new(
                                                rule_category!(),
                                                None,
                                                markup! {
                                                    "Adding a foreign key constraint requires a table scan and locks on both tables."
                                                },
                                            ).detail(None, "This will block writes to both the referencing and referenced tables while PostgreSQL verifies the constraint.")
                                            .note("Add the constraint as NOT VALID first, then VALIDATE it in a separate transaction."));
                                        }
                                    }
                                }
                            }
                        }
                        pgt_query::protobuf::AlterTableType::AtAddColumn => {
                            if let Some(def) = cmd.def.as_ref().and_then(|d| d.node.as_ref()) {
                                if let pgt_query::NodeEnum::ColumnDef(col_def) = def {
                                    // check constraints on the column
                                    for constraint in &col_def.constraints {
                                        if let Some(pgt_query::NodeEnum::Constraint(constr)) =
                                            &constraint.node
                                        {
                                            if constr.contype()
                                                == pgt_query::protobuf::ConstrType::ConstrForeign
                                                && !constr.skip_validation
                                            {
                                                diagnostics.push(RuleDiagnostic::new(
                                                    rule_category!(),
                                                    None,
                                                    markup! {
                                                        "Adding a column with a foreign key constraint requires a table scan and locks."
                                                    },
                                                ).detail(None, "Using REFERENCES when adding a column will block writes while verifying the constraint.")
                                                .note("Add the column without the constraint first, then add the constraint as NOT VALID and VALIDATE it separately."));
                                            }
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
