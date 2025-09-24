use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a primary key constraint results in locks and table rewrites.
    ///
    /// When you add a PRIMARY KEY constraint, PostgreSQL needs to scan the entire table
    /// to verify uniqueness and build the underlying index. This requires an ACCESS EXCLUSIVE
    /// lock which blocks all reads and writes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ADD PRIMARY KEY (id);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE items ADD COLUMN id SERIAL PRIMARY KEY;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- First, create a unique index concurrently
    /// CREATE UNIQUE INDEX CONCURRENTLY items_pk ON items (id);
    /// -- Then add the primary key using the index
    /// ALTER TABLE items ADD CONSTRAINT items_pk PRIMARY KEY USING INDEX items_pk;
    /// ```
    ///
    pub AddingPrimaryKeyConstraint {
        version: "next",
        name: "addingPrimaryKeyConstraint",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("adding-serial-primary-key-field")],
    }
}

impl Rule for AddingPrimaryKeyConstraint {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    match cmd.subtype() {
                        // Check for ADD CONSTRAINT PRIMARY KEY
                        pgls_query::protobuf::AlterTableType::AtAddConstraint => {
                            if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                                cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                if let Some(diagnostic) =
                                    check_for_primary_key_constraint(constraint)
                                {
                                    diagnostics.push(diagnostic);
                                }
                            }
                        }
                        // Check for ADD COLUMN with PRIMARY KEY
                        pgls_query::protobuf::AlterTableType::AtAddColumn => {
                            if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                for constraint in &col_def.constraints {
                                    if let Some(pgls_query::NodeEnum::Constraint(constr)) =
                                        &constraint.node
                                    {
                                        if let Some(diagnostic) =
                                            check_for_primary_key_constraint(constr)
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

fn check_for_primary_key_constraint(
    constraint: &pgls_query::protobuf::Constraint,
) -> Option<RuleDiagnostic> {
    if constraint.contype() == pgls_query::protobuf::ConstrType::ConstrPrimary
        && constraint.indexname.is_empty()
    {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Adding a PRIMARY KEY constraint results in locks and table rewrites."
                },
            )
            .detail(None, "Adding a PRIMARY KEY constraint requires an ACCESS EXCLUSIVE lock which blocks reads.")
            .note("Add the PRIMARY KEY constraint USING an index."),
        )
    } else {
        None
    }
}
