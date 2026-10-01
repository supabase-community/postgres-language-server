use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{FindingKind, ReturnMismatch};
use pgls_catalog::typing::format_type;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// The final statement of a SQL function doesn't return what the function is declared to
    /// return.
    ///
    /// When it creates a `LANGUAGE sql` function, Postgres checks the final statement of the
    /// body against the declared result. A scalar result needs exactly one column. A composite
    /// result, like a table's row type or `RETURNS TABLE`, needs one column per attribute, or a
    /// single column holding the whole row. Unless the function returns `void`, the final
    /// statement must be a `SELECT`, or an `INSERT`, `UPDATE`, `DELETE`, or `MERGE` with
    /// `RETURNING`.
    ///
    /// Postgres raises `42P13 invalid_function_definition` for these functions, unless
    /// `check_function_bodies` is off. The rule only compares the number of columns, not their
    /// types.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id int8, name text, email text);
    /// create function find_user(user_id int8) returns users language sql as $$
    ///     select id, name from users where id = user_id;
    /// $$;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table users (id int8, name text, email text);
    /// create function find_user(user_id int8) returns users language sql as $$
    ///     select * from users where id = user_id;
    /// $$;
    /// ```
    ///
    pub FunctionReturnTypeMismatch {
        version: "next",
        name: "functionReturnTypeMismatch",
        severity: Severity::Error,
        recommended: true,
    }
}

impl LinterRule for FunctionReturnTypeMismatch {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        if !ctx.session().check_function_bodies() {
            return Vec::new();
        }
        let Some(resolution) = ctx.resolution() else {
            return Vec::new();
        };

        resolution
            .findings
            .iter()
            .filter_map(|finding| match &finding.kind {
                FindingKind::FunctionReturnMismatch { declared, mismatch } => {
                    let detail = match mismatch {
                        ReturnMismatch::ColumnCount { expected: 1, .. } => {
                            "Final statement must return exactly one column.".to_string()
                        }
                        ReturnMismatch::ColumnCount { expected, found } if found < expected => {
                            format!(
                                "Final statement returns too few columns: expected {expected}, found {found}."
                            )
                        }
                        ReturnMismatch::ColumnCount { expected, found } => format!(
                            "Final statement returns too many columns: expected {expected}, found {found}."
                        ),
                        ReturnMismatch::NoRows => "Function's final statement must be SELECT or INSERT/UPDATE/DELETE/MERGE RETURNING.".to_string(),
                        ReturnMismatch::ColumnType {
                            position,
                            expected,
                            found,
                        } => {
                            let catalog = ctx.catalog();
                            let (Some(expected), Some(found)) = (
                                format_type(catalog, expected),
                                format_type(catalog, found),
                            ) else {
                                return None;
                            };
                            match position {
                                Some(position) => format!(
                                    "Final statement returns {found} instead of {expected} at column {position}."
                                ),
                                None => format!("Actual return type is {found}."),
                            }
                        }
                    };
                    Some(
                        LinterDiagnostic::new(
                            rule_category!(),
                            finding.span,
                            markup! { "Return type mismatch in function declared to return "<Emphasis>{declared}</Emphasis>"." },
                        )
                        .note(markup! { {detail} }),
                    )
                }
                _ => None,
            })
            .collect()
    }
}
