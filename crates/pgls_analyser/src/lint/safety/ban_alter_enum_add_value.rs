use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `ALTER TYPE ... ADD VALUE` cannot run inside a transaction block in older PostgreSQL versions.
    ///
    /// Adding a value to an enum type acquires an `ACCESS EXCLUSIVE` lock on the enum type.
    /// In PostgreSQL versions before 12, `ALTER TYPE ... ADD VALUE` cannot be executed inside a
    /// transaction block. Even in newer versions, the new value cannot be used in the same
    /// transaction until it is committed.
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
                        <Emphasis>"ALTER TYPE ... ADD VALUE"</Emphasis>" acquires an "<Emphasis>"ACCESS EXCLUSIVE"</Emphasis>" lock on the enum type."
                    },
                )
                .detail(
                    None,
                    "The new enum value cannot be used in the same transaction. In PostgreSQL versions before 12, this statement cannot run inside a transaction block at all.",
                ),
            );
        }

        diagnostics
    }
}
