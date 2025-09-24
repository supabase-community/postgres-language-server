use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding constraints without NOT VALID blocks all reads and writes.
    ///
    /// When adding a CHECK or FOREIGN KEY constraint, PostgreSQL must validate all existing rows,
    /// which requires a full table scan. This blocks reads and writes for the duration.
    ///
    /// Instead, add the constraint with NOT VALID first, then VALIDATE CONSTRAINT in a separate
    /// transaction. This allows reads and writes to continue while validation happens.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address) NOT VALID;
    /// ```
    ///
    pub ConstraintMissingNotValid {
        version: "next",
        name: "constraintMissingNotValid",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("constraint-missing-not-valid")],
    }
}

impl Rule for ConstraintMissingNotValid {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() else {
            return diagnostics;
        };

        for cmd in &stmt.cmds {
            let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node else {
                continue;
            };

            let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                cmd.def.as_ref().and_then(|d| d.node.as_ref())
            else {
                continue;
            };

            if let Some(diagnostic) = check_constraint_needs_not_valid(constraint) {
                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }
}

fn check_constraint_needs_not_valid(
    constraint: &pgls_query::protobuf::Constraint,
) -> Option<RuleDiagnostic> {
    // Skip if the constraint has NOT VALID
    if !constraint.initially_valid {
        return None;
    }

    // Only warn for CHECK and FOREIGN KEY constraints
    match constraint.contype() {
        pgls_query::protobuf::ConstrType::ConstrCheck
        | pgls_query::protobuf::ConstrType::ConstrForeign => Some(
            RuleDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Adding a constraint without NOT VALID will block reads and writes while validating existing rows."
                }
            )
            .detail(None, "Add the constraint as NOT VALID in one transaction, then run VALIDATE CONSTRAINT in a separate transaction.")
        ),
        _ => None,
    }
}
