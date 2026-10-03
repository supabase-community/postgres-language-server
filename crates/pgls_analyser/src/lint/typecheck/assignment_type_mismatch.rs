use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::FindingKind;
use pgls_catalog::typing::format_type;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// An expression assigned to a column cannot be coerced to that column's type. The rule
    /// needs a database connection to load the table and type catalog.
    ///
    /// Postgres reports SQLSTATE `42804` (`datatype_mismatch`).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table typecheck_assignment (qty integer);
    /// insert into typecheck_assignment values (timestamp '2020-01-01');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table typecheck_assignment (qty integer);
    /// insert into typecheck_assignment values (1);
    /// ```
    ///
    pub AssignmentTypeMismatch {
        version: "next",
        name: "assignmentTypeMismatch",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for AssignmentTypeMismatch {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };
        resolution.findings.iter().filter_map(|finding| {
            let FindingKind::AssignmentMismatch { column, column_type, expr_type } = &finding.kind else { return None };
            let catalog = ctx.catalog();
            let (Some(column_type), Some(expr_type)) = (format_type(catalog, column_type), format_type(catalog, expr_type)) else { return None };
            Some(LinterDiagnostic::new(rule_category!(), finding.span, format!("Column \"{column}\" is of type {column_type} but expression is of type {expr_type}."))
                .note("You will need to rewrite or cast the expression."))
        }).collect()
    }
}
