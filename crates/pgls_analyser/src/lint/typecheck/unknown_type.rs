use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A type does not exist.
    ///
    /// Types are looked up in the connected database and in the types created earlier in the same
    /// file, following the search path.
    ///
    /// Postgres raises `42704 undefined_object` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select null::missing_type;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create type mood as enum ('happy', 'sad');
    /// select null::mood;
    /// ```
    ///
    pub UnknownType {
        version: "0.27.0",
        name: "unknownType",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for UnknownType {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::UnknownType { schema, name } => {
                    let name = match schema {
                        Some(schema) => format!("{schema}.{name}"),
                        None => name.clone(),
                    };
                    Some(LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        markup! { "Type "<Emphasis>{name}</Emphasis>" does not exist." },
                    ))
                }
                _ => None,
            })
            .collect()
    }
}
