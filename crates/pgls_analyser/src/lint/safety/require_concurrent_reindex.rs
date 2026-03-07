use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `REINDEX` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock on the table.
    ///
    /// This blocks all reads and writes until the reindex completes. Use `REINDEX CONCURRENTLY`
    /// to rebuild the index without blocking concurrent operations.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// reindex index my_index;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// reindex index concurrently my_index;
    /// ```
    ///
    pub RequireConcurrentReindex {
        version: "next",
        name: "requireConcurrentReindex",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("reindex-non-concurrent")],
    }
}

impl LinterRule for RequireConcurrentReindex {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::ReindexStmt(stmt) = &ctx.stmt() {
            let is_concurrent = stmt.params.iter().any(|param| {
                if let Some(pgls_query::NodeEnum::DefElem(def)) = &param.node {
                    return def.defname == "concurrently";
                }
                false
            });

            if !is_concurrent {
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            <Emphasis>"REINDEX"</Emphasis>" without "<Emphasis>"CONCURRENTLY"</Emphasis>" blocks all table access."
                        },
                    )
                    .detail(
                        None,
                        "Use REINDEX CONCURRENTLY to rebuild the index without blocking reads and writes.",
                    ),
                );
            }
        }

        diagnostics
    }
}
