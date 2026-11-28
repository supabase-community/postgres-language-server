use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableType, ObjectType};

declare_lint_rule! {
    /// Changing the options of a view may change behaviour for existing clients.
    ///
    /// View options such as `security_barrier`, `security_invoker` and `check_option` decide
    /// with whose privileges the view reads its tables, which rows it returns, and which writes
    /// through it Postgres accepts.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter view active_users set (security_invoker = true);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter view active_users reset (security_barrier);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter view active_users rename to current_users;
    /// ```
    ///
    pub BanAlterViewOptions {
        version: "next",
        name: "banAlterViewOptions",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-view-options")],
    }
}

impl LinterRule for BanAlterViewOptions {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() else {
            return vec![];
        };
        if stmt.objtype() != ObjectType::ObjectView {
            return vec![];
        }

        stmt.commands()
            .filter(|cmd| {
                matches!(
                    cmd.subtype(),
                    AlterTableType::AtSetRelOptions | AlterTableType::AtResetRelOptions
                )
            })
            .map(|_| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Changing the options of a view may change behaviour for existing clients."
                    },
                )
                .detail(
                    None,
                    "The options decide with whose privileges the view runs, which rows it returns, and which writes through it Postgres accepts.",
                )
                .note("Check that existing clients work with the new options before changing them.")
            })
            .collect()
    }
}
