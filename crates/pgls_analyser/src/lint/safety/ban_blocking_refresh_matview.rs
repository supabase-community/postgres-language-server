use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `REFRESH MATERIALIZED VIEW` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock.
    ///
    /// This blocks all reads on the materialized view until the refresh completes.
    /// Use `REFRESH MATERIALIZED VIEW CONCURRENTLY` to allow reads during the refresh.
    /// Note: concurrent refresh requires a unique index on the materialized view.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// refresh materialized view my_view;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// refresh materialized view concurrently my_view;
    /// ```
    ///
    pub BanBlockingRefreshMatview {
        version: "next",
        name: "banBlockingRefreshMatview",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("refresh-matview-blocking")],
    }
}

impl LinterRule for BanBlockingRefreshMatview {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::RefreshMatViewStmt(stmt) = &ctx.stmt()
            && !stmt.concurrent
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        <Emphasis>"REFRESH MATERIALIZED VIEW"</Emphasis>" without "<Emphasis>"CONCURRENTLY"</Emphasis>" blocks all reads."
                    },
                )
                .detail(
                    None,
                    "Use REFRESH MATERIALIZED VIEW CONCURRENTLY to allow reads during the refresh. This requires a unique index on the view.",
                ),
            );
        }

        diagnostics
    }
}
