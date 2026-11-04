use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An `INSERT` or `UPDATE` assigns a value to a generated column.
    ///
    /// Postgres computes the value of a generated column (`GENERATED ALWAYS AS (...) STORED`, or
    /// `VIRTUAL` on Postgres 18) itself. `INSERT` can only give it `DEFAULT`, in every row of a
    /// `VALUES` list, and `UPDATE` or `ON CONFLICT DO UPDATE` can only set it to `DEFAULT`.
    ///
    /// Postgres raises `428C9 generated_always` for these statements.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table items (price int, quantity int, total int generated always as (price * quantity) stored);
    /// insert into items (price, quantity, total) values (2, 3, 6);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// create table items (price int, quantity int, total int generated always as (price * quantity) stored);
    /// update items set total = 0;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table items (price int, quantity int, total int generated always as (price * quantity) stored);
    /// insert into items (price, quantity) values (2, 3);
    /// insert into items (price, quantity, total) values (2, 3, default);
    /// ```
    ///
    pub GeneratedColumnAssignment {
        version: "next",
        name: "generatedColumnAssignment",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for GeneratedColumnAssignment {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::GeneratedColumnAssignment { column, insert } => {
                    let diagnostic = if *insert {
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Cannot insert a non-DEFAULT value into column "<Emphasis>{column}</Emphasis>"." },
                        )
                    } else {
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Column "<Emphasis>{column}</Emphasis>" can only be updated to DEFAULT." },
                        )
                    };
                    Some(
                        diagnostic
                            .detail(None, markup! { "Column "<Emphasis>{column}</Emphasis>" is a generated column." })
                            .note("Omit the column, or set it to DEFAULT."),
                    )
                }
                _ => None,
            })
            .collect()
    }
}
