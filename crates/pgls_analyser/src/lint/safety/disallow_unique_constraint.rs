use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
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
        version: "next",
        name: "disallowUniqueConstraint",
        severity: Severity::Error,
        recommended: false,
        sources: &[RuleSource::Squawk("disallow-unique-constraint")],
    }
}

impl Rule for DisallowUniqueConstraint {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            // Check if this table was created in the same transaction
            let table_name = stmt.relation.as_ref().map(|r| &r.relname);

            // Look for tables created in previous statements of this file
            let table_created_in_transaction = if let Some(table_name) = table_name {
                ctx.file_context().previous_stmts().iter().any(|prev_stmt| {
                    if let pgls_query::NodeEnum::CreateStmt(create) = prev_stmt {
                        create
                            .relation
                            .as_ref()
                            .is_some_and(|r| &r.relname == table_name)
                    } else {
                        false
                    }
                })
            } else {
                false
            };

            if !table_created_in_transaction {
                for cmd in &stmt.cmds {
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                        match cmd.subtype() {
                            pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                                if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                                    &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                                {
                                    // Check if it's a unique constraint without an existing index
                                    if constraint.contype()
                                        == pgls_query::protobuf::ConstrType::ConstrUnique
                                        && constraint.indexname.is_empty()
                                    {
                                        diagnostics.push(
                                            RuleDiagnostic::new(
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
                                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                    &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                                {
                                    // Check for inline unique constraints
                                    for constraint in &col_def.constraints {
                                        if let Some(pgls_query::NodeEnum::Constraint(constr)) =
                                            &constraint.node
                                        {
                                            if constr.contype()
                                                == pgls_query::protobuf::ConstrType::ConstrUnique
                                            {
                                                diagnostics.push(
                                                    RuleDiagnostic::new(
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
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
