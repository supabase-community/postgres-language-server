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
    /// This also applies to exclusion constraints defined inline in `CREATE TABLE`.
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
    pub BanAddExclusionConstraint {
        version: "next",
        name: "banAddExclusionConstraint",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("add-constraint-exclude")],
    }
}

impl LinterRule for BanAddExclusionConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        match &ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(stmt) => {
                for cmd in &stmt.cmds {
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                        && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddConstraint
                    {
                        if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                            cmd.def.as_ref().and_then(|d| d.node.as_ref())
                        {
                            if constraint.contype()
                                == pgls_query::protobuf::ConstrType::ConstrExclusion
                            {
                                diagnostics.push(exclusion_diagnostic());
                            }
                        }
                    }
                }
            }
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                for constraint_node in &stmt.constraints {
                    if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                        &constraint_node.node
                    {
                        if constraint.contype() == pgls_query::protobuf::ConstrType::ConstrExclusion
                        {
                            diagnostics.push(exclusion_diagnostic());
                        }
                    }
                }
            }
            _ => {}
        }

        diagnostics
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
