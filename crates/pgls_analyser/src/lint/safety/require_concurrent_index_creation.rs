use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Creating indexes non-concurrently can lock the table for writes.
    ///
    /// When creating an index on an existing table, using CREATE INDEX without CONCURRENTLY will lock the table
    /// against writes for the duration of the index build. This can cause downtime in production systems.
    /// Use CREATE INDEX CONCURRENTLY to build the index without blocking concurrent operations.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE INDEX users_email_idx ON users (email);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
    /// ```
    ///
    pub RequireConcurrentIndexCreation {
        version: "next",
        name: "requireConcurrentIndexCreation",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("require-concurrent-index-creation")],
    }
}

impl Rule for RequireConcurrentIndexCreation {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        let pgls_query::NodeEnum::IndexStmt(stmt) = &ctx.stmt() else {
            return diagnostics;
        };

        // Concurrent indexes are safe
        if stmt.concurrent {
            return diagnostics;
        }

        // Check if this table was created in the same transaction/file
        let table_name = stmt
            .relation
            .as_ref()
            .map(|r| r.relname.as_str())
            .unwrap_or("");

        // Skip if table name is empty or table was created in the same file
        if table_name.is_empty() || is_table_created_in_file(ctx.file_context(), table_name) {
            return diagnostics;
        }

        diagnostics.push(
            RuleDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Creating an index non-concurrently blocks writes to the table."
                },
            )
            .detail(None, "Use CREATE INDEX CONCURRENTLY to avoid blocking concurrent operations on the table.")
        );

        diagnostics
    }
}

fn is_table_created_in_file(
    file_context: &pgls_analyse::AnalysedFileContext,
    table_name: &str,
) -> bool {
    // Check all statements in the file to see if this table was created
    for stmt in file_context.stmts {
        if let pgls_query::NodeEnum::CreateStmt(create_stmt) = stmt {
            if let Some(relation) = &create_stmt.relation {
                if relation.relname == table_name {
                    return true;
                }
            }
        }
    }
    false
}
