use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a policy or rule may change behaviour for existing clients.
    ///
    /// Dropping a row level security policy changes which rows existing clients can read and
    /// write. Dropping a rule changes what their queries on the table or view do.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop policy if exists tenant_isolation on accounts;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// drop rule if exists protect_accounts on accounts;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create policy tenant_isolation on accounts using (tenant_id = current_setting('app.tenant_id')::int);
    /// ```
    ///
    pub BanDropPolicy {
        version: "next",
        name: "banDropPolicy",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-policy")],
    }
}

impl LinterRule for BanDropPolicy {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        let (message, detail) = match stmt.remove_type() {
            ObjectType::ObjectPolicy => (
                "Dropping a policy may change access for existing clients.",
                "Existing clients may be able to read or write different rows once the policy is gone.",
            ),
            ObjectType::ObjectRule => (
                "Dropping a rule may change behaviour for existing clients.",
                "Postgres no longer rewrites the queries of existing clients on the table or view.",
            ),
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(rule_category!(), None, markup! { {message} })
                .detail(None, detail)
                .note("Update clients that depend on it before dropping it."),
        ]
    }
}
