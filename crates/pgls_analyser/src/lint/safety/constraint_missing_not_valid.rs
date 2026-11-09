use crate::fix::StatementTokens;
use crate::utils::{ConstraintSite, added_constraints};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding constraints without NOT VALID blocks all reads and writes.
    ///
    /// When adding a CHECK, FOREIGN KEY or (on Postgres 18) NOT NULL constraint, Postgres must
    /// validate all existing rows, which requires a full table scan. This blocks reads and writes
    /// for the duration.
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
        version: "0.15.0",
        name: "constraintMissingNotValid",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("constraint-missing-not-valid")],
    }
}

impl LinterRule for ConstraintMissingNotValid {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        added_constraints(ctx.stmt())
            .filter(|(_, site)| *site == ConstraintSite::Table)
            .filter_map(|(constraint, _)| {
                let diagnostic = check_constraint_needs_not_valid(constraint)?;
                Some(diagnostic.maybe_fix(add_not_valid(ctx.tokens(), constraint)))
            })
            .collect()
    }
}

fn check_constraint_needs_not_valid(
    constraint: &pgls_query::protobuf::Constraint,
) -> Option<LinterDiagnostic> {
    // Skip if the constraint has NOT VALID
    if !constraint.initially_valid {
        return None;
    }

    // Only warn for the constraints that can be NOT VALID: CHECK, FOREIGN KEY, and NOT NULL
    // (`ADD CONSTRAINT ... NOT NULL column`, Postgres 18)
    match constraint.contype() {
        pgls_query::protobuf::ConstrType::ConstrCheck
        | pgls_query::protobuf::ConstrType::ConstrForeign
        | pgls_query::protobuf::ConstrType::ConstrNotnull => Some(
            LinterDiagnostic::new(
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

/// Appends `NOT VALID` to the constraint. It is unsafe because the existing rows are not
/// checked until the constraint is validated.
fn add_not_valid(
    tokens: Option<StatementTokens>,
    constraint: &pgls_query::protobuf::Constraint,
) -> Option<RuleFix> {
    let tokens = tokens?;
    let start = tokens.at(constraint.location)?;
    let end = tokens.element_end(start);
    Some(tokens.insert_after(end, "Add NOT VALID", "NOT VALID"))
}
