use crate::utils::dotted_name;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Creating enum types is not recommended for new applications.
    ///
    /// Enumerated types have several limitations that make them difficult to work with in production:
    ///
    /// - Removing values from an enum requires complex migrations and is not supported directly
    /// - Adding values to an enum requires an ACCESS EXCLUSIVE lock in some Postgres versions
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
        version: "0.17.0",
        name: "creatingEnum",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Eugene("W13")],
    }
}

impl LinterRule for CreatingEnum {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::CreateEnumStmt(stmt) = &ctx.stmt() {
            let type_name = dotted_name(&stmt.type_name);

            diagnostics.push(
                LinterDiagnostic::new(
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
