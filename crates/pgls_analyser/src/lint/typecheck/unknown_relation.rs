use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// A table, view, or materialized view does not exist.
    ///
    /// The relation is looked up in the connected database and in the relations created earlier in
    /// the same file, following the search path of the session (including `SET search_path` in the
    /// file). Temporary tables are found in `pg_temp`.
    ///
    /// Postgres raises `42P01 undefined_table` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select * from missing_table;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create temp table scratch (id int8);
    /// select * from scratch;
    /// ```
    ///
    pub UnknownRelation {
        version: "0.27.0",
        name: "unknownRelation",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for UnknownRelation {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::UnknownRelation { schema, name } => {
                    let name = match schema {
                        Some(schema) => format!("{schema}.{name}"),
                        None => name.clone(),
                    };
                    Some(LinterDiagnostic::new(
                        rule_category!(),
                        finding.span,
                        markup! { "Relation "<Emphasis>{name}</Emphasis>" does not exist." },
                    ))
                }
                _ => None,
            })
            .collect()
    }
}
