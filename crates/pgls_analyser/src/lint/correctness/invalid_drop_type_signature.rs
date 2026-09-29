use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `DROP TYPE` and `DROP DOMAIN` don't take a parameter list.
    ///
    /// Unlike `DROP FUNCTION`, types are dropped by name only. Postgres parses the parameter list
    /// as type modifiers and raises `42601 syntax_error` ("type modifier is not allowed").
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// drop type if exists group_composite (int, text);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// drop type if exists group_composite;
    /// ```
    ///
    pub InvalidDropTypeSignature {
        version: "next",
        name: "invalidDropTypeSignature",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for InvalidDropTypeSignature {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::DropStmt(stmt) = ctx.stmt() else {
            return Vec::new();
        };
        if !matches!(
            stmt.remove_type(),
            pgls_query::protobuf::ObjectType::ObjectType
                | pgls_query::protobuf::ObjectType::ObjectDomain
        ) {
            return Vec::new();
        }

        stmt.objects
            .iter()
            .filter_map(|object| match &object.node {
                Some(pgls_query::NodeEnum::TypeName(type_name))
                    if !type_name.typmods.is_empty() =>
                {
                    Some(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! { "Types are dropped by name only, without a parameter list." },
                        )
                        .detail(None, "Remove the parameter list."),
                    )
                }
                _ => None,
            })
            .collect()
    }
}
