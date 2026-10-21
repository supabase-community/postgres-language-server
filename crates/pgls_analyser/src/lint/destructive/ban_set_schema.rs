use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, linter_context::object_type_noun};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Moving an object to another schema may break existing clients.
    ///
    /// Clients that use the old schema-qualified name, like `public.users`, fail after the
    /// object moved. Rolling back the application does not move the object back.
    ///
    /// Update clients to use the new name, and keep the object at its old name until no running
    /// application revision depends on it.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table users set schema auth;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter function calculate_total(int) set schema billing;
    /// ```
    ///
    pub BanSetSchema {
        version: "next",
        name: "banSetSchema",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-set-schema")],
    }
}

impl LinterRule for BanSetSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterObjectSchemaStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        let object = object_type_noun(stmt.object_type());
        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Moving "{object}" to another schema may break existing clients."
                },
            )
            .detail(
                None,
                "Clients that use the old schema-qualified name fail, and rolling back the application does not move it back.",
            )
            .note("Update clients to use the new schema-qualified name before moving the object."),
        ]
    }
}
