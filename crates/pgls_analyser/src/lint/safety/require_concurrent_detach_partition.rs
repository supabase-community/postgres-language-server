use crate::utils::{alter_table_cmds, cmd_def};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Detaching a partition without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock.
    ///
    /// `ALTER TABLE ... DETACH PARTITION` without `CONCURRENTLY` blocks all reads and writes
    /// on the parent table. Use `DETACH PARTITION ... CONCURRENTLY` (Postgres 14+) to
    /// avoid blocking concurrent operations.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table my_table detach partition my_partition;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table my_table detach partition my_partition concurrently;
    /// ```
    ///
    pub RequireConcurrentDetachPartition {
        version: "0.25.0",
        name: "requireConcurrentDetachPartition",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Pgfence("detach-partition")],
    }
}

impl LinterRule for RequireConcurrentDetachPartition {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        for cmd in alter_table_cmds(ctx.stmt()) {
            if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtDetachPartition
                && !matches!(
                    cmd_def(cmd),
                    Some(pgls_query::NodeEnum::PartitionCmd(p)) if p.concurrent
                )
            {
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Detaching a partition without "<Emphasis>"CONCURRENTLY"</Emphasis>" blocks all table access."
                        },
                    )
                    .detail(
                        None,
                        "Use DETACH PARTITION ... CONCURRENTLY (Postgres 14+) to avoid blocking reads and writes.",
                    ),
                );
            }
        }

        diagnostics
    }
}
