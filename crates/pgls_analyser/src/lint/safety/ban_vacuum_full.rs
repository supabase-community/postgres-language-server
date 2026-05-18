use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, linter_context::is_vacuum_full};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `VACUUM FULL` rewrites the entire table and acquires an `ACCESS EXCLUSIVE` lock.
    ///
    /// This blocks all reads and writes for the duration of the operation, which can
    /// take a very long time on large tables. Use regular `VACUUM` or `pg_repack` instead
    /// for online table maintenance.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// vacuum full my_table;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// vacuum my_table;
    /// ```
    ///
    pub BanVacuumFull {
        version: "next",
        name: "banVacuumFull",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Pgfence("vacuum-full")],
    }
}

impl LinterRule for BanVacuumFull {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::VacuumStmt(stmt) = &ctx.stmt() {
            if is_vacuum_full(stmt) {
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            <Emphasis>"VACUUM FULL"</Emphasis>" rewrites the entire table and blocks all access."
                        },
                    )
                    .detail(
                        None,
                        "Use regular VACUUM or pg_repack for online table maintenance without blocking reads and writes.",
                    ),
                );
            }
        }

        diagnostics
    }
}
