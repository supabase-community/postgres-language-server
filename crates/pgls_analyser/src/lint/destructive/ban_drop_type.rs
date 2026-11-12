use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, utils::object_type_noun};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Dropping a type, cast, operator, operator class, or operator family may break existing clients.
    ///
    /// Casts and parameters that name a dropped type fail with `42704 undefined_object`. With
    /// `CASCADE`, columns of that type are dropped too.
    ///
    /// Update your application code to no longer use the type, then drop it in a later migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop type order_status;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// drop cast (text as point);
    /// ```
    ///
    pub BanDropType {
        version: "next",
        name: "banDropType",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-type")],
    }
}

impl LinterRule for BanDropType {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        let object_type = stmt.remove_type();
        if !matches!(
            object_type,
            ObjectType::ObjectType
                | ObjectType::ObjectCast
                | ObjectType::ObjectOperator
                | ObjectType::ObjectOpclass
                | ObjectType::ObjectOpfamily
        ) {
            return vec![];
        }
        let kind = object_type_noun(object_type);

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Dropping "{kind}" may break existing clients."
                },
            )
            .detail(
                None,
                "Queries that still use it fail, and rolling back the application does not restore it.",
            )
            .note("Update your application code to no longer use it, then drop it in a later migration."),
        ]
    }
}
