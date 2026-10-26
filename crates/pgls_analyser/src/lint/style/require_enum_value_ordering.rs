use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `ALTER TYPE ... ADD VALUE` without `BEFORE` or `AFTER` appends the value to the end of the enum.
    ///
    /// Enum values are ordered, and queries like `where status < 'approved'` depend on that
    /// order. Appending a value that belongs in the middle silently changes what those queries
    /// return.
    ///
    /// Say where the new value belongs with `BEFORE` or `AFTER`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter type order_status add value 'in_review';
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter type order_status add value 'in_review' before 'approved';
    /// ```
    ///
    pub RequireEnumValueOrdering {
        version: "next",
        name: "requireEnumValueOrdering",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("require-enum-value-ordering")],
    }
}

impl LinterRule for RequireEnumValueOrdering {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if let pgls_query::NodeEnum::AlterEnumStmt(stmt) = ctx.stmt()
            && stmt.old_val.is_empty()
            && !stmt.new_val.is_empty()
            && stmt.new_val_neighbor.is_empty()
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Enum value "<Emphasis>{stmt.new_val}</Emphasis>" is appended to the end of the enum."
                    },
                )
                .detail(
                    None,
                    "Queries that compare enum values depend on their order.",
                )
                .note("Add BEFORE or AFTER to specify the position of the new value."),
            ];
        }

        vec![]
    }
}
