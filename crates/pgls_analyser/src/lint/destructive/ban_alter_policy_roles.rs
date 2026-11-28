use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Changing the roles of a policy may change access for existing clients.
    ///
    /// A row level security policy only applies to the roles in its `TO` clause. Changing them
    /// may take access away from the roles of existing clients, or apply the policy to roles it
    /// didn't apply to before.
    ///
    /// Changing the condition of a policy is reported by `banAlterPolicyCondition`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter policy tenant_isolation on accounts to authenticated;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter policy tenant_isolation on accounts rename to accounts_tenant_isolation;
    /// ```
    ///
    pub BanAlterPolicyRoles {
        version: "next",
        name: "banAlterPolicyRoles",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-policy-roles")],
    }
}

impl LinterRule for BanAlterPolicyRoles {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterPolicyStmt(stmt) = ctx.stmt() else {
            return vec![];
        };
        if stmt.roles.is_empty() {
            return vec![];
        }

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Changing the roles of policy "<Emphasis>{stmt.policy_name}</Emphasis>" may change access for existing clients."
                },
            )
            .detail(
                None,
                "The policy no longer applies to the roles it applied to before, or applies to new ones.",
            )
            .note("Check which roles existing clients use before changing the roles of the policy."),
        ]
    }
}
