use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a sequence may break existing clients.
    ///
    /// An earlier application revision may still call `nextval('my_seq')` after the migration
    /// dropped the sequence. Rolling back the application does not restore the sequence, so that
    /// call fails. With `CASCADE`, column defaults that use the sequence are dropped too.
    ///
    /// Update clients to stop using the sequence, and drop it in a later migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop sequence order_number_seq;
    /// ```
    ///
    pub BanDropSequence {
        version: "next",
        name: "banDropSequence",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-sequence")],
    }
}

impl LinterRule for BanDropSequence {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt()
            && stmt.remove_type() == ObjectType::ObjectSequence
        {
            return vec![
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping a sequence may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Clients that still call nextval() on the sequence fail, and rolling back the application does not restore it.",
                )
                .note("Update your application code to stop using the sequence, then drop it in a later migration."),
            ];
        }

        vec![]
    }
}
