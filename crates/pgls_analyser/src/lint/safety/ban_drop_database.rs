use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

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

impl LinterRule for BanDropDatabase {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::DropdbStmt(_) = &ctx.stmt() {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping a database may break existing clients."
                    },
                )
                .detail(None, "You probably don't want to drop your database."),
            );
        }

        diagnostics
    }
}
