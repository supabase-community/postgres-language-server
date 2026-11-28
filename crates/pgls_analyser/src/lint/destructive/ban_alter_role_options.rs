use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;
use pgls_query::protobuf::DefElem;

declare_lint_rule! {
    /// Changing the options or settings of a role may change access or behaviour for existing
    /// clients.
    ///
    /// Role options such as `LOGIN`, `BYPASSRLS` or `CONNECTION LIMIT` decide whether clients
    /// can connect as the role and what they can access. `SET` and `RESET` change the settings
    /// of their sessions, e.g. the `search_path`, including for a single database with
    /// `IN DATABASE`. `ALTER USER` is an alias of `ALTER ROLE` and is reported too.
    ///
    /// Renaming a role and changing the members of a group with `ALTER GROUP` are not reported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter role app_user nologin;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter user app_user in database app_db set search_path to private;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter role app_user rename to api_user;
    /// ```
    ///
    pub BanAlterRoleOptions {
        version: "next",
        name: "banAlterRoleOptions",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-role-options")],
    }
}

impl LinterRule for BanAlterRoleOptions {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let (message, detail) = match ctx.stmt() {
            // `ALTER GROUP ... ADD|DROP USER` is an `AlterRoleStmt` with only `rolemembers`.
            NodeEnum::AlterRoleStmt(stmt)
                if DefElem::find(&stmt.options, "rolemembers").is_none() =>
            {
                (
                    "Changing the options of a role may change access for existing clients.",
                    "The options decide whether clients can connect as the role and what they can access.",
                )
            }
            NodeEnum::AlterRoleSetStmt(_) => (
                "Changing the settings of a role may change behaviour for existing clients.",
                "The settings apply to every new session of the role, e.g. which schemas its queries find.",
            ),
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(rule_category!(), None, markup! { {message} })
                .detail(None, detail)
                .note("Check that the clients that use the role work with the change before making it."),
        ]
    }
}
