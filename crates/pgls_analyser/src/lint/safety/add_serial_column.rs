use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a column with a SERIAL type or GENERATED ALWAYS AS ... STORED causes a full table rewrite.
    ///
    /// When adding a column with a SERIAL type (serial, bigserial, smallserial) or a GENERATED ALWAYS AS ... STORED column
    /// to an existing table, PostgreSQL must rewrite the entire table while holding an ACCESS EXCLUSIVE lock.
    /// This blocks all reads and writes to the table for the duration of the rewrite operation.
    ///
    /// SERIAL types are implemented using sequences and DEFAULT values, while GENERATED ... STORED columns require
    /// computing and storing values for all existing rows. Both operations require rewriting every row in the table.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE prices ADD COLUMN id serial;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE prices ADD COLUMN id bigserial;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE prices ADD COLUMN total int GENERATED ALWAYS AS (price * quantity) STORED;
    /// ```
    ///
    pub AddSerialColumn {
        version: "next",
        name: "addSerialColumn",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Eugene("E11")],
    }
}

impl Rule for AddSerialColumn {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn {
                        if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                            &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                        {
                            // Check for SERIAL types
                            if let Some(type_name) = &col_def.type_name {
                                let type_str = get_type_name(type_name);
                                if is_serial_type(&type_str) {
                                    diagnostics.push(
                                        RuleDiagnostic::new(
                                            rule_category!(),
                                            None,
                                            markup! {
                                                "Adding a column with type "<Emphasis>{type_str}</Emphasis>" requires a table rewrite."
                                            },
                                        )
                                        .detail(None, "SERIAL types require rewriting the entire table with an ACCESS EXCLUSIVE lock, blocking all reads and writes.")
                                        .note("SERIAL types cannot be added to existing tables without a full table rewrite. Consider using a non-serial type with a sequence instead."),
                                    );
                                    continue;
                                }
                            }

                            // Check for GENERATED ALWAYS AS ... STORED
                            let has_stored_generated =
                                col_def.constraints.iter().any(|constraint| {
                                    if let Some(pgls_query::NodeEnum::Constraint(c)) =
                                        &constraint.node
                                    {
                                        c.contype()
                                            == pgls_query::protobuf::ConstrType::ConstrGenerated
                                            && c.generated_when == "a" // 'a' = ALWAYS
                                    } else {
                                        false
                                    }
                                });

                            if has_stored_generated {
                                diagnostics.push(
                                    RuleDiagnostic::new(
                                        rule_category!(),
                                        None,
                                        markup! {
                                            "Adding a column with "<Emphasis>"GENERATED ALWAYS AS ... STORED"</Emphasis>" requires a table rewrite."
                                        },
                                    )
                                    .detail(None, "GENERATED ... STORED columns require rewriting the entire table with an ACCESS EXCLUSIVE lock, blocking all reads and writes.")
                                    .note("GENERATED ... STORED columns cannot be added to existing tables without a full table rewrite."),
                                );
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

fn get_type_name(type_name: &pgls_query::protobuf::TypeName) -> String {
    type_name
        .names
        .iter()
        .filter_map(|n| {
            if let Some(pgls_query::NodeEnum::String(s)) = &n.node {
                Some(s.sval.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn is_serial_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "serial"
            | "bigserial"
            | "smallserial"
            | "pg_catalog.serial"
            | "pg_catalog.bigserial"
            | "pg_catalog.smallserial"
            // Also check for serial2, serial4, serial8 which are aliases
            | "serial2"
            | "serial4"
            | "serial8"
            | "pg_catalog.serial2"
            | "pg_catalog.serial4"
            | "pg_catalog.serial8"
    )
}
