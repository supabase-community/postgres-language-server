use crate::utils::{builtin_type_name, column_defs, dotted_name};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer BIGINT over smaller integer types.
    ///
    /// Using smaller integer types like SMALLINT, INTEGER, or their aliases can lead to overflow
    /// issues as your application grows. BIGINT provides a much larger range and helps avoid
    /// future migration issues when values exceed the limits of smaller types.
    ///
    /// The storage difference between INTEGER (4 bytes) and BIGINT (8 bytes) is minimal on
    /// modern systems, while the cost of migrating to a larger type later can be significant.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     id integer
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     id serial
    /// );
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     id bigint
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     id bigserial
    /// );
    /// ```
    ///
    pub PreferBigInt {
        version: "0.15.0",
        name: "preferBigInt",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-big-int")],
    }
}

#[derive(
    Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferBigIntOptions {
    pub check_int: bool,
    pub check_smallint: bool,
}

impl Default for PreferBigIntOptions {
    fn default() -> Self {
        Self {
            check_int: true,
            check_smallint: true,
        }
    }
}

impl LinterRule for PreferBigInt {
    type Options = PreferBigIntOptions;

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();
        for col_def in column_defs(ctx.stmt()) {
            check_column_def(&mut diagnostics, col_def, ctx.options());
        }
        diagnostics
    }
}

fn check_column_def(
    diagnostics: &mut Vec<LinterDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
    options: &PreferBigIntOptions,
) {
    let Some(type_name) = &col_def.type_name else {
        return;
    };
    // A serial type becomes an integer of its size. Otherwise only the built-in integers count,
    // not types of the user with the same name.
    let (name, integer_type) = match pgls_catalog::serial_type(type_name) {
        Some(serial) => (dotted_name(&type_name.names), serial.integer_type()),
        None => match builtin_type_name(type_name) {
            Some(name) => (name.to_owned(), name),
            None => return,
        },
    };
    let is_smallint = integer_type == "int2";
    let is_int = integer_type == "int4";

    if (is_smallint && options.check_smallint) || (is_int && options.check_int) {
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Using smaller integer types can lead to overflow issues."
                },
            )
            .detail(
                None,
                format!(
                    "The '{name}' type has a limited range that may be exceeded as your data grows."
                ),
            )
            .note("Consider using BIGINT for integer columns to avoid future migration issues."),
        );
    }
}
