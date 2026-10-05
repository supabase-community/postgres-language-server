use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// No function with this name accepts this number of arguments.
    ///
    /// Functions are looked up in the connected database and in the functions created earlier in
    /// the same file, following the search path. Default arguments and `VARIADIC` parameters are
    /// taken into account. Argument types are not checked yet.
    ///
    /// Postgres raises `42883 undefined_function` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select missing_function();
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create function add_one(value int8) returns int8 language sql as 'select value + 1';
    /// select add_one(1);
    /// ```
    ///
    pub UnknownFunction {
        version: "0.27.0",
        name: "unknownFunction",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for UnknownFunction {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::UnknownFunction {
                    schema,
                    name,
                    arg_count,
                    name_exists,
                } => {
                    let name = match schema {
                        Some(schema) => format!("{schema}.{name}"),
                        None => name.clone(),
                    };
                    Some(if *name_exists {
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "No overload of "<Emphasis>{name}</Emphasis>" accepts "{arg_count}" arguments." },
                        )
                    } else {
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Function "<Emphasis>{name}</Emphasis>" does not exist." },
                        )
                    })
                }
                _ => None,
            })
            .collect()
    }
}
