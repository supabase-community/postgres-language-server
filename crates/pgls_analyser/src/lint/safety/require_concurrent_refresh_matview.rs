use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
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
    pub RequireConcurrentRefreshMatview {
        version: "0.25.0",
        name: "requireConcurrentRefreshMatview",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Pgfence("refresh-matview-blocking")],
    }
}

impl LinterRule for RequireConcurrentRefreshMatview {
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
                )
                .maybe_fix(add_concurrently(ctx, stmt)),
            );
        }

        diagnostics
    }
}

/// `CONCURRENTLY` can't be combined with `WITH NO DATA`. It is unsafe because it fails without a
/// unique index on the view.
fn add_concurrently(
    ctx: &LinterRuleContext<RequireConcurrentRefreshMatview>,
    stmt: &pgls_query::protobuf::RefreshMatViewStmt,
) -> Option<RuleFix> {
    if stmt.skip_data {
        return None;
    }
    let tokens = ctx.tokens()?;
    let view = tokens.find(pgls_query::protobuf::Token::View)?;
    Some(tokens.insert_after(view, "Add CONCURRENTLY", "CONCURRENTLY"))
}
