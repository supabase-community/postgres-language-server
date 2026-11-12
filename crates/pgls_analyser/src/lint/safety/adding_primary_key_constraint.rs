use crate::utils::{added_constraints, altered_relation};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a primary key constraint results in locks and table rewrites.
    ///
    /// When you add a PRIMARY KEY constraint, Postgres needs to scan the entire table
    /// to verify uniqueness and build the underlying index. This requires an ACCESS EXCLUSIVE
    /// lock which blocks all reads and writes.
    ///
    /// Instead, create a unique index concurrently and add the primary key using that index. A
    /// temporal primary key (`WITHOUT OVERLAPS`, Postgres 18) needs a GiST index, which can't be
    /// unique, so it has no such alternative.
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
        version: "0.15.0",
        name: "addingPrimaryKeyConstraint",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("adding-serial-primary-key-field")],
    }
}

impl LinterRule for AddingPrimaryKeyConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // Other transactions can't see a table created earlier in the transaction, so its
        // lock blocks no one.
        if altered_relation(ctx.stmt()).is_some_and(|relation| ctx.is_new_relation(relation)) {
            return vec![];
        }
        // `ADD CONSTRAINT ... PRIMARY KEY`, and `ADD COLUMN ... PRIMARY KEY`.
        added_constraints(ctx.stmt())
            .filter_map(|(constraint, _)| check_for_primary_key_constraint(constraint))
            .collect()
    }
}

fn check_for_primary_key_constraint(
    constraint: &pgls_query::protobuf::Constraint,
) -> Option<LinterDiagnostic> {
    if constraint.contype() == pgls_query::protobuf::ConstrType::ConstrPrimary
        && constraint.indexname.is_empty()
    {
        // `USING INDEX` needs a unique B-tree index, but a temporal key needs a GiST index.
        let note = if constraint.without_overlaps {
            "There is no concurrent alternative for a temporal key (WITHOUT OVERLAPS). Use SET lock_timeout to limit the impact on concurrent operations."
        } else {
            "Add the PRIMARY KEY constraint USING an index."
        };
        Some(
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Adding a PRIMARY KEY constraint results in locks and table rewrites."
                },
            )
            .detail(None, "Adding a PRIMARY KEY constraint requires an ACCESS EXCLUSIVE lock which blocks reads.")
            .note(note),
        )
    } else {
        None
    }
}
