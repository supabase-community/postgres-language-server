use crate::utils::{added_constraints, altered_relation};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding an exclusion constraint acquires an `ACCESS EXCLUSIVE` lock.
    ///
    /// Exclusion constraints require a full table scan to validate and block all reads
    /// and writes while held. Unlike other constraints, there is no concurrent alternative.
    /// Use `SET lock_timeout` to limit the impact on concurrent operations.
    ///
    /// Exclusion constraints in `CREATE TABLE` are not reported: the new table is empty, and no
    /// other session can use it before the transaction commits.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table my_table add constraint my_excl exclude using gist (col with &&);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table my_table add constraint my_check check (col > 0) not valid;
    /// ```
    ///
    pub AvoidAddingExclusionConstraint {
        version: "0.25.0",
        name: "avoidAddingExclusionConstraint",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Pgfence("add-constraint-exclude")],
    }
}

impl LinterRule for AvoidAddingExclusionConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // Other transactions can't see a table created earlier in the transaction, so its
        // lock blocks no one.
        if altered_relation(ctx.stmt()).is_some_and(|relation| ctx.is_new_relation(relation)) {
            return vec![];
        }
        // An exclusion constraint is always a table constraint, never inline on a column.
        added_constraints(ctx.stmt())
            .filter(|(constraint, _)| {
                constraint.contype() == pgls_query::protobuf::ConstrType::ConstrExclusion
            })
            .map(|_| exclusion_diagnostic())
            .collect()
    }
}

fn exclusion_diagnostic() -> LinterDiagnostic {
    LinterDiagnostic::new(
        rule_category!(),
        None,
        markup! {
            "Adding an exclusion constraint acquires an "<Emphasis>"ACCESS EXCLUSIVE"</Emphasis>" lock."
        },
    )
    .detail(
        None,
        "There is no concurrent alternative for exclusion constraints. Use SET lock_timeout to limit the impact on concurrent operations.",
    )
}
