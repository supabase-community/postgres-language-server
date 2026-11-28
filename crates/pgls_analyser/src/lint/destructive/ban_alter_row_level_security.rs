use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::AlterTableType;

declare_lint_rule! {
    /// Changing row level security on a table may change access for existing clients.
    ///
    /// Enabling row level security hides every row from clients that no policy lets through,
    /// and disabling it gives them access to all rows again. `FORCE` and `NO FORCE` decide
    /// whether the policies apply to the owner of the table too.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table accounts enable row level security;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter table accounts no force row level security;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table accounts add column tenant_id int;
    /// ```
    ///
    pub BanAlterRowLevelSecurity {
        version: "next",
        name: "banAlterRowLevelSecurity",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-row-level-security")],
    }
}

impl LinterRule for BanAlterRowLevelSecurity {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        ctx.stmt().alter_table_commands()
            .filter(|cmd| {
                matches!(
                    cmd.subtype(),
                    AlterTableType::AtEnableRowSecurity
                        | AlterTableType::AtDisableRowSecurity
                        | AlterTableType::AtForceRowSecurity
                        | AlterTableType::AtNoForceRowSecurity
                )
            })
            .map(|_| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Changing row level security may change access for existing clients."
                    },
                )
                .detail(
                    None,
                    "Existing clients may no longer see rows that no policy lets through, or see rows the policies hid.",
                )
                .note("Create the policies that existing clients need before enabling row level security, and check who relies on it before disabling it.")
            })
            .collect()
    }
}
