use crate::utils::alter_table_cmds;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::alter_table_cmd_lock;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Attaching a partition acquires an `ACCESS EXCLUSIVE` lock on the partition.
    ///
    /// `ALTER TABLE ... ATTACH PARTITION` takes a `SHARE UPDATE EXCLUSIVE` lock on the parent
    /// table, which doesn't block reads and writes, and an `ACCESS EXCLUSIVE` lock on the
    /// partition and on the default partition, if there is one. While holding them, Postgres
    /// scans the partition to check that its rows match the partition bound, and the default
    /// partition to check that none of its rows do. For large tables, this can cause significant
    /// downtime. Add a `CHECK` constraint that matches the partition bound to the partition
    /// before attaching it, so that Postgres skips the scan.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub AvoidAttachingPartition {
        version: "0.25.0",
        name: "avoidAttachingPartition",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Pgfence("attach-partition")],
    }
}

impl LinterRule for AvoidAttachingPartition {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        for cmd in alter_table_cmds(ctx.stmt()) {
            if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAttachPartition {
                let parent_lock = alter_table_cmd_lock(cmd).name();
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Attaching a partition acquires an "<Emphasis>"ACCESS EXCLUSIVE"</Emphasis>" lock on the partition and a "<Emphasis>{parent_lock}</Emphasis>" lock on the parent table."
                        },
                    )
                    .detail(
                        None,
                        "This blocks all reads and writes on the partition, and on the default partition if there is one, while Postgres scans them to validate the partition bound. Add a CHECK constraint that matches the partition bound to the partition before attaching it, so that Postgres skips the scan.",
                    ),
                );
            }
        }

        diagnostics
    }
}
