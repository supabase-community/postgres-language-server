use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a view or materialized view may break existing clients.
    ///
    /// Queries that still select from the view fail with `42P01 undefined_table`. Rolling back
    /// the application does not bring the view back.
    ///
    /// Update your application code to no longer use the view, then drop it in a later migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop view active_users;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// drop materialized view user_stats;
    /// ```
    ///
    pub BanDropView {
        version: "next",
        name: "banDropView",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-view")],
    }
}

impl LinterRule for BanDropView {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        let kind = match stmt.remove_type() {
            ObjectType::ObjectView => "a view",
            ObjectType::ObjectMatview => "a materialized view",
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Dropping "{kind}" may break existing clients."
                },
            )
            .detail(
                None,
                "Queries that still use the view fail, and rolling back the application does not restore it.",
            )
            .note("Update your application code to no longer use the view, then drop it in a later migration."),
        ]
    }
}
