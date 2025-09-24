use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
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
        version: "next",
        name: "preferBigInt",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-big-int")],
    }
}

impl Rule for PreferBigInt {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        match &ctx.stmt() {
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                for table_elt in &stmt.table_elts {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                        check_column_def(&mut diagnostics, col_def);
                    }
                }
            }
            pgls_query::NodeEnum::AlterTableStmt(stmt) => {
                for cmd in &stmt.cmds {
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                        if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                            || cmd.subtype()
                                == pgls_query::protobuf::AlterTableType::AtAlterColumnType
                        {
                            if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                check_column_def(&mut diagnostics, col_def);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        diagnostics
    }
}

fn check_column_def(
    diagnostics: &mut Vec<RuleDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
) {
    if let Some(type_name) = &col_def.type_name {
        for name_node in &type_name.names {
            if let Some(pgls_query::NodeEnum::String(name)) = &name_node.node {
                let type_name_lower = name.sval.to_lowercase();
                let is_small_int = matches!(
                    type_name_lower.as_str(),
                    "smallint"
                        | "integer"
                        | "int2"
                        | "int4"
                        | "serial"
                        | "serial2"
                        | "serial4"
                        | "smallserial"
                );

                if is_small_int {
                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Using smaller integer types can lead to overflow issues."
                            },
                        )
                        .detail(None, format!("The '{}' type has a limited range that may be exceeded as your data grows.", name.sval))
                        .note("Consider using BIGINT for integer columns to avoid future migration issues."),
                    );
                }
            }
        }
    }
}
