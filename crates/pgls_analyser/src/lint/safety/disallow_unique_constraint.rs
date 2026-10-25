use crate::utils::{alter_table_cmds, cmd_def};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Disallow adding a UNIQUE constraint without using an existing index.
    ///
    /// Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock, which blocks all reads and
    /// writes to the table. Instead, create a unique index concurrently and then add the
    /// constraint using that index.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE table_name ADD CONSTRAINT field_name_constraint UNIQUE (field_name);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE foo ADD COLUMN bar text UNIQUE;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE UNIQUE INDEX CONCURRENTLY dist_id_temp_idx ON distributors (dist_id);
    /// ALTER TABLE distributors DROP CONSTRAINT distributors_pkey,
    /// ADD CONSTRAINT distributors_pkey PRIMARY KEY USING INDEX dist_id_temp_idx;
    /// ```
    ///
    pub DisallowUniqueConstraint {
        version: "0.15.0",
        name: "disallowUniqueConstraint",
        severity: Severity::Error,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("disallow-unique-constraint")],
    }
}

impl LinterRule for DisallowUniqueConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            // Check if this table was created earlier in the transaction
            let table_created_in_transaction = stmt.relation.as_ref().is_some_and(|relation| {
                ctx.session()
                    .has_created_object(&relation.schemaname, &relation.relname)
            });

            if !table_created_in_transaction {
                for cmd in alter_table_cmds(ctx.stmt()) {
                    match cmd.subtype() {
                        pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                            if let Some(pgls_query::NodeEnum::Constraint(constraint)) = cmd_def(cmd)
                            {
                                // Check if it's a unique constraint without an existing index
                                if constraint.contype()
                                    == pgls_query::protobuf::ConstrType::ConstrUnique
                                    && constraint.indexname.is_empty()
                                {
                                    diagnostics.push(
                                        LinterDiagnostic::new(
                                            rule_category!(),
                                            None,
                                            markup! {
                                                "Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock."
                                            },
                                        )
                                        .note("Create a unique index CONCURRENTLY and then add the constraint using that index."),
                                    );
                                }
                            }
                        }
                        pgls_query::protobuf::AlterTableType::AtAddColumn => {
                            if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = cmd_def(cmd) {
                                // Check for inline unique constraints
                                for constraint in &col_def.constraints {
                                    if let Some(pgls_query::NodeEnum::Constraint(constr)) =
                                        &constraint.node
                                        && constr.contype()
                                            == pgls_query::protobuf::ConstrType::ConstrUnique
                                    {
                                        diagnostics.push(
                                                LinterDiagnostic::new(
                                                    rule_category!(),
                                                    None,
                                                    markup! {
                                                        "Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock."
                                                    },
                                                )
                                                .note("Create a unique index CONCURRENTLY and then add the constraint using that index."),
                                            );
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
