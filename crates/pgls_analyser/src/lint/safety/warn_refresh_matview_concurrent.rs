use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `REFRESH MATERIALIZED VIEW CONCURRENTLY` still acquires an `EXCLUSIVE` lock.
    ///
    /// While concurrent refresh allows reads during the refresh, it still blocks DDL
    /// and other write operations on the materialized view. On large views, this can
    /// take a long time.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// refresh materialized view concurrently my_view;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub WarnRefreshMatviewConcurrent {
        version: "next",
        name: "warnRefreshMatviewConcurrent",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("refresh-matview-concurrent")],
    }
}

impl LinterRule for WarnRefreshMatviewConcurrent {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::RefreshMatViewStmt(stmt) = &ctx.stmt()
            && stmt.concurrent
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        <Emphasis>"REFRESH MATERIALIZED VIEW CONCURRENTLY"</Emphasis>" still acquires an "<Emphasis>"EXCLUSIVE"</Emphasis>" lock."
                    },
                )
                .detail(
                    None,
                    "Concurrent refresh allows reads but still blocks DDL and writes. Consider the impact on long-running refreshes.",
                ),
            );
        }

        diagnostics
    }
}
