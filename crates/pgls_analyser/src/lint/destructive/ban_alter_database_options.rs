use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;
use pgls_query::protobuf::DefElem;

declare_lint_rule! {
    /// Changing the options or settings of a database may change behaviour for existing clients.
    ///
    /// Database options such as `ALLOW_CONNECTIONS` or `CONNECTION LIMIT` decide whether clients
    /// can connect. `SET` and `RESET` change the settings of every new session in the database,
    /// e.g. the `search_path` or the time zone.
    ///
    /// Renaming the database, changing its owner or its tablespace, and refreshing its collation
    /// version are not reported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter database app set search_path to app, public;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter database app with connection limit 5;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter database app rename to app_v2;
    /// ```
    ///
    pub BanAlterDatabaseOptions {
        version: "next",
        name: "banAlterDatabaseOptions",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-database-options")],
    }
}

impl LinterRule for BanAlterDatabaseOptions {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let (message, detail) = match ctx.stmt() {
            // `SET TABLESPACE` is an `AlterDatabaseStmt` with only `tablespace`.
            NodeEnum::AlterDatabaseStmt(stmt)
                if DefElem::find(&stmt.options, "tablespace").is_none() =>
            {
                (
                    "Changing the options of a database may change behaviour for existing clients.",
                    "The options decide whether and how many clients can connect to the database.",
                )
            }
            NodeEnum::AlterDatabaseSetStmt(_) => (
                "Changing the settings of a database may change behaviour for existing clients.",
                "The settings apply to every new session in the database, e.g. which schemas its queries find.",
            ),
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(rule_category!(), None, markup! { {message} })
                .detail(None, detail)
                .note(
                    "Check that the clients of the database work with the change before making it.",
                ),
        ]
    }
}
