use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dropping a trigger acquires an `ACCESS EXCLUSIVE` lock on the table.
    ///
    /// `DROP TRIGGER` blocks all reads and writes on the table while the lock is held.
    /// It may also break application logic that depends on the trigger's behavior.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop trigger my_trigger on my_table;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanDropTrigger {
        version: "next",
        name: "banDropTrigger",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("drop-trigger")],
    }
}

impl LinterRule for BanDropTrigger {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::DropStmt(stmt) = &ctx.stmt()
            && stmt.remove_type() == pgls_query::protobuf::ObjectType::ObjectTrigger
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping a trigger acquires an "<Emphasis>"ACCESS EXCLUSIVE"</Emphasis>" lock on the table."
                    },
                )
                .detail(
                    None,
                    "This blocks all reads and writes. Ensure no application logic depends on the trigger before dropping it.",
                ),
            );
        }

        diagnostics
    }
}
