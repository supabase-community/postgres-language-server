use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Creating a trigger acquires a `SHARE ROW EXCLUSIVE` lock on the table.
    ///
    /// `CREATE TRIGGER` can block concurrent writes while the lock is held.
    /// Triggers also add hidden complexity to write operations on the table,
    /// which can cause unexpected performance issues and make debugging harder.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create trigger my_trigger after insert on my_table for each row execute function my_func();
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanCreateTrigger {
        version: "next",
        name: "banCreateTrigger",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("create-trigger")],
    }
}

impl LinterRule for BanCreateTrigger {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::CreateTrigStmt(_) = &ctx.stmt() {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Creating a trigger acquires a "<Emphasis>"SHARE ROW EXCLUSIVE"</Emphasis>" lock on the table."
                    },
                )
                .detail(
                    None,
                    "Triggers add hidden complexity and can block concurrent writes. Consider using application-level logic instead.",
                ),
            );
        }

        diagnostics
    }
}
