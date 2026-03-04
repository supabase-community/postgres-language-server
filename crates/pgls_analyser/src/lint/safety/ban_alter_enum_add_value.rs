use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `ALTER TYPE ... ADD VALUE` cannot run inside a transaction block in older Postgres versions.
    ///
    /// In Postgres versions before 12, `ALTER TYPE ... ADD VALUE` cannot be executed inside a
    /// transaction block at all. On Postgres 12+, the operation is fast (metadata-only), but the
    /// new enum value cannot be used in the same transaction until it is committed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter type my_enum add value 'new_value';
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanAlterEnumAddValue {
        version: "next",
        name: "banAlterEnumAddValue",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Pgfence("alter-enum-add-value")],
    }
}

impl LinterRule for BanAlterEnumAddValue {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::AlterEnumStmt(_) = &ctx.stmt() {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        <Emphasis>"ALTER TYPE ... ADD VALUE"</Emphasis>" cannot be used in a transaction block before Postgres 12."
                    },
                )
                .detail(
                    None,
                    "On Postgres 12+, the operation is fast but the new value cannot be used in the same transaction until committed.",
                ),
            );
        }

        diagnostics
    }
}
