use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping an extension may break existing clients.
    ///
    /// `DROP EXTENSION` also removes the functions, types, operators and other objects of the
    /// extension, which existing clients may depend on.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop extension hstore;
    /// ```
    ///
    pub BanDropExtension {
        version: "next",
        name: "banDropExtension",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-extension")],
    }
}

impl LinterRule for BanDropExtension {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt()
            && stmt.remove_type() == ObjectType::ObjectExtension
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping an extension may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Dropping an extension also removes its functions, types, and other objects.",
                )
                .note("Update clients to no longer use the objects of the extension before dropping it."),
            ];
        }

        vec![]
    }
}
