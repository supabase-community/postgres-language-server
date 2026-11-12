use crate::utils::added_constraints;
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
    /// A temporal unique constraint (`WITHOUT OVERLAPS`, Postgres 18) needs a GiST index, which
    /// can't be unique, so it has no such alternative.
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
        let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() else {
            return vec![];
        };
        // A table created earlier in the transaction is not visible to other sessions yet.
        if stmt
            .relation
            .as_ref()
            .is_some_and(|relation| ctx.session().has_created_relation(relation, ctx.catalog()))
        {
            return vec![];
        }

        // Inline constraints of `ADD COLUMN` can't name an index (`USING INDEX`) or be temporal
        // (`WITHOUT OVERLAPS`).
        added_constraints(ctx.stmt())
            .filter(|(constraint, _)| {
                constraint.contype() == pgls_query::protobuf::ConstrType::ConstrUnique
                    && constraint.indexname.is_empty()
            })
            .map(|(constraint, _)| {
                // `USING INDEX` needs a unique B-tree index, but a temporal key needs a GiST
                // index.
                let note = if constraint.without_overlaps {
                    "There is no concurrent alternative for a temporal key (WITHOUT OVERLAPS). Use SET lock_timeout to limit the impact on concurrent operations."
                } else {
                    "Create a unique index CONCURRENTLY and then add the constraint using that index."
                };
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock."
                    },
                )
                .note(note)
            })
            .collect()
    }
}
