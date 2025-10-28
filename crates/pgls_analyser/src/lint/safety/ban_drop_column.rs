use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dropping a column may break existing clients.
    ///
    /// Update your application code to no longer read or write the column.
    ///
    /// You can leave the column as nullable or delete the column once queries no longer select or modify the column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table test drop column id;
    /// ```
    ///
    pub BanDropColumn {
        version: "next",
        name: "banDropColumn",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("ban-drop-column")],
    }
}

impl Rule for BanDropColumn {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtDropColumn {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Dropping a column may break existing clients."
                            },
                        ).detail(None, "You can leave the column as nullable or delete the column once queries no longer select or modify the column."));
                    }
                }
            }
        }

        diagnostics
    }
}
