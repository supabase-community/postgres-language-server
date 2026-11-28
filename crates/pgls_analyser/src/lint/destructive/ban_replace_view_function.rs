use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Replacing a view, function, procedure, trigger, rule or aggregate may change behaviour for
    /// existing clients.
    ///
    /// `CREATE OR REPLACE` changes what existing clients get back or what their statements do,
    /// without any change to their SQL.
    ///
    /// `CREATE OR REPLACE AGGREGATE` is only checked on Postgres 12 and later, and
    /// `CREATE OR REPLACE TRIGGER` on Postgres 14 and later, which added them.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create or replace view active_users as select id, email from users where active;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// create or replace function total(a int, b int) returns int language sql as $$ select a + b $$;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create view active_users as select id, email from users where active;
    /// ```
    ///
    pub BanReplaceViewFunction {
        version: "next",
        name: "banReplaceViewFunction",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-replace-view-function")],
    }
}

impl LinterRule for BanReplaceViewFunction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let catalog = ctx.catalog();
        let object = match ctx.stmt() {
            NodeEnum::ViewStmt(stmt) if stmt.replace => "a view",
            NodeEnum::CreateFunctionStmt(stmt) if stmt.replace && stmt.is_procedure => {
                "a procedure"
            }
            NodeEnum::CreateFunctionStmt(stmt) if stmt.replace => "a function",
            NodeEnum::CreateTrigStmt(stmt)
                if stmt.replace && catalog.server_version_at_least(14) != Some(false) =>
            {
                "a trigger"
            }
            NodeEnum::RuleStmt(stmt) if stmt.replace => "a rule",
            NodeEnum::DefineStmt(stmt)
                if stmt.replace
                    && stmt.kind() == ObjectType::ObjectAggregate
                    && catalog.server_version_at_least(12) != Some(false) =>
            {
                "an aggregate"
            }
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Replacing "{object}" may change behaviour for existing clients."
                },
            )
            .detail(
                None,
                "Existing clients get different results or side effects without any change to their SQL.",
            )
            .note("Check that existing clients work with the new definition, or create a new object and move clients to it."),
        ]
    }
}
