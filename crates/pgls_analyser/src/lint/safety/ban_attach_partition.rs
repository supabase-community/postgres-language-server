use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Attaching a partition acquires an `ACCESS EXCLUSIVE` lock on the parent table.
    ///
    /// `ALTER TABLE ... ATTACH PARTITION` locks the parent table, blocking all reads and writes.
    /// For large tables, this can cause significant downtime. Consider creating the partition
    /// with the correct constraints upfront, or use a staging table approach.
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
    pub BanAttachPartition {
        version: "next",
        name: "banAttachPartition",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("attach-partition")],
    }
}

impl LinterRule for BanAttachPartition {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAttachPartition
                {
                    diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Attaching a partition acquires an "<Emphasis>"ACCESS EXCLUSIVE"</Emphasis>" lock on the parent table."
                            },
                        )
                        .detail(
                            None,
                            "This blocks all reads and writes on the parent table. Consider adding a matching CHECK constraint to the child table before attaching to minimize lock duration.",
                        ),
                    );
                }
            }
        }

        diagnostics
    }
}
