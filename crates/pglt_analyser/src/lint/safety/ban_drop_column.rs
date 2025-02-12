use pglt_analyse::{context::RuleContext, declare_lint_rule, Rule, RuleDiagnostic, RuleSource};
use pglt_console::markup;

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
    /// ```sql,ignore
    /// alter table test drop column id;
    /// ```
    ///
    pub BanDropColumn {
        version: "next",
        name: "banDropColumn",
        recommended: true,
        sources: &[RuleSource::Squawk("ban-drop-column")],
    }
}

impl Rule for BanDropColumn {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pglt_query_ext::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pglt_query_ext::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pglt_query_ext::protobuf::AlterTableType::AtDropColumn {
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
