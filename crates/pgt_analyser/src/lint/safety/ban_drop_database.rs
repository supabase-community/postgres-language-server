use pgt_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgt_console::markup;
use pgt_diagnostics::Severity;

declare_lint_rule! {
    /// Dropping a database may break existing clients (and everything else, really).
    ///
    /// Make sure that you really want to drop it.
    pub BanDropDatabase {
        version: "next",
        name: "banDropDatabase",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("ban-drop-database")],
    }
}

impl Rule for BanDropDatabase {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = vec![];

        if let pgt_query_ext::NodeEnum::DropStmt(stmt) = &ctx.stmt() {
            if stmt.remove_type() == pgt_query_ext::protobuf::ObjectType::ObjectDatabase {
                diagnostics.push(
                    RuleDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Dropping a database may break existing clients."
                        },
                    )
                    .detail(None, "You probably don't want to drop your database."),
                );
            }
        }

        diagnostics
    }
}
