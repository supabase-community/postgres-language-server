use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dropping a schema will remove all objects within it and may break existing clients.
    ///
    /// A `DROP SCHEMA` statement removes the entire schema and all objects it contains.
    /// This is a destructive operation that can cause significant data loss and break
    /// dependent applications.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop schema my_schema;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub BanDropSchema {
        version: "next",
        name: "banDropSchema",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Pgfence("drop-schema")],
    }
}

impl LinterRule for BanDropSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::DropStmt(stmt) = &ctx.stmt()
            && stmt.remove_type() == pgls_query::protobuf::ObjectType::ObjectSchema
        {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping a schema will remove all objects within it and may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Remove objects individually instead, or ensure all dependent applications have been updated.",
                ),
            );
        }

        diagnostics
    }
}
