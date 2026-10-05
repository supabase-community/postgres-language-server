use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A schema does not exist.
    ///
    /// `pg_catalog` and `pg_temp` always exist. Schemas created earlier in the same file are known.
    ///
    /// Postgres raises `3F000 invalid_schema_name` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select * from missing_schema.users;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create schema app;
    /// create table app.users (id int8);
    /// select * from app.users;
    /// ```
    ///
    pub UnknownSchema {
        version: "0.27.0",
        name: "unknownSchema",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for UnknownSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::UnknownSchema { name } => Some(LinterDiagnostic::new(
                    rule_category!(),
                    finding.span,
                    markup! { "Schema "<Emphasis>{name}</Emphasis>" does not exist." },
                )),
                _ => None,
            })
            .collect()
    }
}
