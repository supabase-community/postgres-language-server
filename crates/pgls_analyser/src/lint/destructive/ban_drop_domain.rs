use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a domain may break existing clients.
    ///
    /// Clients that use the domain in casts or parameters fail. With `CASCADE`, columns of the
    /// domain are dropped too.
    ///
    /// Move clients to a replacement type before dropping the domain.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop domain email;
    /// ```
    ///
    pub BanDropDomain {
        version: "next",
        name: "banDropDomain",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-domain")],
    }
}

impl LinterRule for BanDropDomain {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt()
            && stmt.remove_type() == ObjectType::ObjectDomain
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping a domain may break existing clients."
                    },
                )
                .detail(None, "Casts and parameters that name the domain fail once it is gone.")
                .note("Move clients to a replacement type, then drop the domain in a later migration."),
            ];
        }

        vec![]
    }
}
