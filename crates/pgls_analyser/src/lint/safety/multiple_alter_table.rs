use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Multiple ALTER TABLE statements on the same table should be combined into a single statement.
    ///
    /// When you run multiple ALTER TABLE statements on the same table, PostgreSQL must scan and potentially
    /// rewrite the table multiple times. Each ALTER TABLE command requires acquiring locks and performing
    /// table operations that can be expensive, especially on large tables.
    ///
    /// Combining multiple ALTER TABLE operations into a single statement with comma-separated actions
    /// allows PostgreSQL to scan and modify the table only once, improving performance and reducing
    /// the time locks are held.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
    /// ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE authors
    ///   ALTER COLUMN name SET NOT NULL,
    ///   ALTER COLUMN email SET NOT NULL;
    /// ```
    ///
    pub MultipleAlterTable {
        version: "next",
        name: "multipleAlterTable",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Eugene("W12")],
    }
}

impl Rule for MultipleAlterTable {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check if current statement is ALTER TABLE
        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            if let Some(relation) = &stmt.relation {
                let current_schema = &relation.schemaname;
                let current_table = &relation.relname;

                // Check previous statements for ALTER TABLE on the same table
                let file_ctx = ctx.file_context();

                // Normalize schema name: treat empty string as "public"
                let current_schema_normalized = if current_schema.is_empty() {
                    "public"
                } else {
                    current_schema.as_str()
                };

                let has_previous_alter =
                    file_ctx
                        .previous_stmts()
                        .iter()
                        .any(|prev_stmt| match prev_stmt {
                            pgls_query::NodeEnum::AlterTableStmt(prev_alter) => {
                                if let Some(prev_relation) = &prev_alter.relation {
                                    let prev_schema_normalized =
                                        if prev_relation.schemaname.is_empty() {
                                            "public"
                                        } else {
                                            prev_relation.schemaname.as_str()
                                        };

                                    // Match if same table and schema (treating empty schema as "public")
                                    prev_relation.relname == *current_table
                                        && prev_schema_normalized == current_schema_normalized
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        });

                if has_previous_alter {
                    let schema_display = if current_schema.is_empty() {
                        "public"
                    } else {
                        current_schema
                    };

                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Multiple "<Emphasis>"ALTER TABLE"</Emphasis>" statements found for table "<Emphasis>{schema_display}"."{{current_table}}</Emphasis>"."
                            },
                        )
                        .detail(
                            None,
                            "Multiple ALTER TABLE statements on the same table require scanning and potentially rewriting the table multiple times.",
                        )
                        .note("Combine the ALTER TABLE statements into a single statement with comma-separated actions to scan the table only once."),
                    );
                }
            }
        }

        diagnostics
    }
}
