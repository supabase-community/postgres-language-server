use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Changing the condition of a policy may change access for existing clients.
    ///
    /// The `USING` and `WITH CHECK` expressions of a row level security policy decide which rows
    /// clients can read and write. Changing them may hide rows from existing clients, reject
    /// their writes, or give them access to rows they couldn't see before.
    ///
    /// Each changed clause is reported. Changing the roles of a policy is reported by
    /// `banAlterPolicyRoles`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter policy tenant_isolation on accounts using (tenant_id = current_setting('app.tenant_id')::int);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter policy tenant_isolation on accounts rename to accounts_tenant_isolation;
    /// ```
    ///
    pub BanAlterPolicyCondition {
        version: "next",
        name: "banAlterPolicyCondition",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-policy-condition")],
    }
}

impl LinterRule for BanAlterPolicyCondition {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterPolicyStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        [("USING", &stmt.qual), ("WITH CHECK", &stmt.with_check)]
            .into_iter()
            .filter(|(_, expr)| expr.is_some())
            .map(|(clause, _)| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Changing the "<Emphasis>{clause}</Emphasis>" condition of policy "<Emphasis>{stmt.policy_name}</Emphasis>" may change access for existing clients."
                    },
                )
                .detail(
                    None,
                    "Existing clients may no longer see or write rows they could before, or see and write rows they couldn't.",
                )
                .note("Check which rows existing clients need before changing the condition.")
            })
            .collect()
    }
}
