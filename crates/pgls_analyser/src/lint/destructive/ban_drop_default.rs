use crate::utils::alter_table_cmds;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::AlterTableType;

declare_lint_rule! {
    /// Dropping a column or domain default may break existing clients.
    ///
    /// Inserts that omit the column fail with `23502 not_null_violation` if the column is
    /// `NOT NULL`, and silently write `NULL` otherwise.
    ///
    /// Update your application code to provide a value for the column, then drop the default in a
    /// later migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table users alter column created_at drop default;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter domain status drop default;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table users alter column created_at set default now();
    /// ```
    ///
    pub BanDropDefault {
        version: "next",
        name: "banDropDefault",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-default")],
    }
}

impl LinterRule for BanDropDefault {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let drops_default = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(_) => alter_table_cmds(ctx.stmt())
                .any(|cmd| cmd.subtype() == AlterTableType::AtColumnDefault && cmd.def.is_none()),
            // 'T' is `SET DEFAULT` or, without a default, `DROP DEFAULT`.
            pgls_query::NodeEnum::AlterDomainStmt(stmt) => {
                stmt.subtype == "T" && stmt.def.is_none()
            }
            _ => false,
        };

        if !drops_default {
            return vec![];
        }

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Dropping a default may break existing clients."
                },
            )
            .detail(
                None,
                "Inserts that omit the column fail if it is NOT NULL, and write NULL otherwise.",
            )
            .note("Update your application code to provide a value, then drop the default in a later migration."),
        ]
    }
}
