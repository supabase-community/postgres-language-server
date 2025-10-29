use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Creating enum types is not recommended for new applications.
    ///
    /// Enumerated types have several limitations that make them difficult to work with in production:
    ///
    /// - Removing values from an enum requires complex migrations and is not supported directly
    /// - Adding values to an enum requires an ACCESS EXCLUSIVE lock in some PostgreSQL versions
    /// - Associating additional data with enum values is impossible without restructuring
    /// - Renaming enum values requires careful migration planning
    ///
    /// A lookup table with a foreign key constraint provides more flexibility and is easier to maintain.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TYPE document_type AS ENUM ('invoice', 'receipt', 'other');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- Use a lookup table instead
    /// CREATE TABLE document_type (
    ///     type_name TEXT PRIMARY KEY
    /// );
    /// INSERT INTO document_type VALUES ('invoice'), ('receipt'), ('other');
    /// ```
    ///
    pub CreatingEnum {
        version: "next",
        name: "creatingEnum",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Eugene("W13")],
    }
}

impl Rule for CreatingEnum {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::CreateEnumStmt(stmt) = &ctx.stmt() {
            let type_name = get_type_name(&stmt.type_name);

            diagnostics.push(
                RuleDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Creating enum type "<Emphasis>{type_name}</Emphasis>" is not recommended."
                    },
                )
                .detail(None, "Enum types are difficult to modify: removing values requires complex migrations, and associating additional data with values is not possible.")
                .note("Consider using a lookup table with a foreign key constraint instead, which provides more flexibility and easier maintenance."),
            );
        }

        diagnostics
    }
}

fn get_type_name(type_name_nodes: &[pgls_query::protobuf::Node]) -> String {
    type_name_nodes
        .iter()
        .filter_map(|n| {
            if let Some(pgls_query::NodeEnum::String(s)) = &n.node {
                Some(s.sval.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}
