use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer TIMESTAMPTZ over TIMESTAMP types.
    ///
    /// Using TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
    /// TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) stores timestamps with time zone information,
    /// making it safer for applications that handle multiple time zones or need to track
    /// when events occurred in absolute time.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE app.users (
    ///     created_ts timestamp
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE app.accounts (
    ///     created_ts timestamp without time zone
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamp;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE app.users (
    ///     created_ts timestamptz
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE app.accounts (
    ///     created_ts timestamp with time zone
    /// );
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamptz;
    /// ```
    ///
    pub PreferTimestamptz {
        version: "next",
        name: "preferTimestamptz",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-timestamptz")],
    }
}

impl Rule for PreferTimestamptz {
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
                        match cmd.subtype() {
                            pgls_query::protobuf::AlterTableType::AtAddColumn
                            | pgls_query::protobuf::AlterTableType::AtAlterColumnType => {
                                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                    &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                                {
                                    check_column_def(&mut diagnostics, col_def);
                                }
                            }
                            _ => {}
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
        if let Some(last_name) = type_name.names.last() {
            if let Some(pgls_query::NodeEnum::String(name)) = &last_name.node {
                // Check for "timestamp" (without timezone)
                if name.sval.to_lowercase() == "timestamp" {
                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Prefer TIMESTAMPTZ over TIMESTAMP for better timezone handling."
                            },
                        )
                        .detail(None, "TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.")
                        .note("Use TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) instead."),
                    );
                }
            }
        }
    }
}
