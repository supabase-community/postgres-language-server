use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Enabling or disabling a trigger acquires a `SHARE ROW EXCLUSIVE` lock.
    ///
    /// `ALTER TABLE ... ENABLE/DISABLE TRIGGER` blocks concurrent writes while the lock is held.
    /// This can cause downtime on busy tables.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table my_table enable trigger my_trigger;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanEnableDisableTrigger {
        version: "next",
        name: "banEnableDisableTrigger",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("enable-disable-trigger")],
    }
}

const TRIGGER_SUBTYPES: &[pgls_query::protobuf::AlterTableType] = &[
    pgls_query::protobuf::AlterTableType::AtEnableTrig,
    pgls_query::protobuf::AlterTableType::AtDisableTrig,
    pgls_query::protobuf::AlterTableType::AtEnableTrigAll,
    pgls_query::protobuf::AlterTableType::AtDisableTrigAll,
    pgls_query::protobuf::AlterTableType::AtEnableTrigUser,
    pgls_query::protobuf::AlterTableType::AtDisableTrigUser,
];

impl LinterRule for BanEnableDisableTrigger {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && TRIGGER_SUBTYPES.contains(&cmd.subtype())
                {
                    diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Enabling or disabling a trigger acquires a "<Emphasis>"SHARE ROW EXCLUSIVE"</Emphasis>" lock."
                            },
                        )
                        .detail(
                            None,
                            "This blocks concurrent writes. Consider the impact on busy tables and use SET lock_timeout.",
                        ),
                    );
                }
            }
        }

        diagnostics
    }
}
