use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
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
            let is_full = stmt.options.iter().any(|opt| {
                if let Some(pgls_query::NodeEnum::DefElem(def)) = &opt.node {
                    if def.defname == "full" {
                        return match &def.arg {
                            Some(arg) => match &arg.node {
                                Some(pgls_query::NodeEnum::Integer(i)) => i.ival != 0,
                                _ => true,
                            },
                            None => true,
                        };
                    }
                }
                false
            });

            if is_full {
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
