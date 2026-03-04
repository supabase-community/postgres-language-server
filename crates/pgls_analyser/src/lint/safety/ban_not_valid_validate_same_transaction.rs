use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Validating a constraint in the same transaction it was added as `NOT VALID` defeats the purpose.
    ///
    /// Adding a constraint with `NOT VALID` avoids a full table scan and lock during creation.
    /// But if you immediately `VALIDATE CONSTRAINT` in the same transaction, the validation
    /// still holds the lock from the `ADD CONSTRAINT`, blocking reads and writes.
    ///
    /// Run `VALIDATE CONSTRAINT` in a separate transaction to get the benefit of `NOT VALID`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// Adding a NOT VALID constraint and validating it in the same transaction:
    /// ```sql
    /// ALTER TABLE orders ADD CONSTRAINT orders_check CHECK (total > 0) NOT VALID;
    /// ALTER TABLE orders VALIDATE CONSTRAINT orders_check;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanNotValidValidateSameTransaction {
        version: "next",
        name: "banNotValidValidateSameTransaction",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Pgfence("not-valid-validate-same-tx")],
    }
}

impl LinterRule for BanNotValidValidateSameTransaction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() else {
            return diagnostics;
        };

        let tx_state = ctx.file_context().transaction_state();

        let (table_schema, table_name) = stmt
            .relation
            .as_ref()
            .map(|r| (r.schemaname.as_str(), r.relname.as_str()))
            .unwrap_or_default();

        for cmd in &stmt.cmds {
            if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtValidateConstraint
                && !cmd.name.is_empty()
                && tx_state.has_not_valid_constraint(table_schema, table_name, &cmd.name)
            {
                let constraint_name = &cmd.name;
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Constraint "<Emphasis>{constraint_name}</Emphasis>" was added as NOT VALID and validated in the same transaction."
                        },
                    )
                    .detail(
                        None,
                        "Run VALIDATE CONSTRAINT in a separate transaction to avoid holding locks during validation.",
                    ),
                );
            }
        }

        diagnostics
    }
}
