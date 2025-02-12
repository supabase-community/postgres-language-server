use pglt_analyse::{context::RuleContext, declare_lint_rule, Rule, RuleDiagnostic, RuleSource};
use pglt_console::markup;

declare_lint_rule! {
    /// Dropping a NOT NULL constraint may break existing clients.
    ///
    /// Application code or code written in procedural languages like PL/SQL or PL/pgSQL may not expect NULL values for the column that was previously guaranteed to be NOT NULL and therefore may fail to process them correctly.
    ///
    /// You can consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,ignore
    /// alter table users alter column email drop not null;
    /// ```
    pub BanDropNotNull {
        version: "next",
        name: "banDropNotNull",
        recommended: true,
        sources: &[RuleSource::Squawk("ban-drop-not-null")],

    }
}

impl Rule for BanDropNotNull {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pglt_query_ext::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pglt_query_ext::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pglt_query_ext::protobuf::AlterTableType::AtDropNotNull {
                        diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Dropping a NOT NULL constraint may break existing clients."
                            },
                        ).detail(None, "Consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values."));
                    }
                }
            }
        }

        diagnostics
    }
}
