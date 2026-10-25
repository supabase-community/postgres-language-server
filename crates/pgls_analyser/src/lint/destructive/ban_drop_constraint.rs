use crate::utils::alter_table_cmds;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::AlterTableType;

declare_lint_rule! {
    /// Dropping a constraint may break existing clients.
    ///
    /// A constraint is a guarantee that clients can depend on. For example, an application that
    /// uses `INSERT ... ON CONFLICT ON CONSTRAINT my_constraint`, or whose `ON CONFLICT (col)`
    /// relies on a unique constraint, fails as soon as the constraint is gone.
    ///
    /// Update clients to no longer depend on the constraint before dropping it.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table users drop constraint users_email_key;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter domain positive_int drop constraint positive_int_check;
    /// ```
    ///
    pub BanDropConstraint {
        version: "next",
        name: "banDropConstraint",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-constraint")],
    }
}

impl LinterRule for BanDropConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let drops_constraint = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(_) => alter_table_cmds(ctx.stmt())
                .any(|cmd| cmd.subtype() == AlterTableType::AtDropConstraint),
            // 'X' is `DROP CONSTRAINT`.
            pgls_query::NodeEnum::AlterDomainStmt(stmt) => stmt.subtype == "X",
            _ => false,
        };

        if !drops_constraint {
            return vec![];
        }

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Dropping a constraint may break existing clients."
                },
            )
            .detail(
                None,
                "Clients may depend on the guarantee of the constraint, e.g. as the arbiter of INSERT ... ON CONFLICT.",
            )
            .note("Update clients to no longer depend on the constraint, then drop it in a later migration."),
        ]
    }
}
