use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::Token;

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
        version: "0.15.0",
        name: "requireConcurrentIndexCreation",
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("require-concurrent-index-creation")],
    }
}

impl LinterRule for RequireConcurrentIndexCreation {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
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
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Creating an index non-concurrently blocks writes to the table."
                },
            )
            .detail(None, "Use CREATE INDEX CONCURRENTLY to avoid blocking concurrent operations on the table.")
            .maybe_fix(add_concurrently(ctx)),
        );

        diagnostics
    }
}

/// `CREATE INDEX CONCURRENTLY` can't run in a transaction block.
fn add_concurrently(ctx: &LinterRuleContext<RequireConcurrentIndexCreation>) -> Option<RuleFix> {
    if ctx.session().transaction_depth() > 0 {
        return None;
    }
    let tokens = ctx.tokens()?;
    let index = tokens.find(Token::Index)?;
    Some(tokens.insert_after(index, "Add CONCURRENTLY", "CONCURRENTLY"))
}

fn is_table_created_in_file(file_context: &crate::AnalysedFileContext, table_name: &str) -> bool {
    // Check all statements in the file to see if this table was created
    for stmt in file_context.stmts {
        if let pgls_query::NodeEnum::CreateStmt(create_stmt) = stmt
            && let Some(relation) = &create_stmt.relation
            && relation.relname == table_name
        {
            return true;
        }
    }
    false
}
