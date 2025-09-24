use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Dropping indexes non-concurrently can lock the table for reads.
    ///
    /// When dropping an index, using DROP INDEX without CONCURRENTLY will lock the table
    /// preventing reads and writes for the duration of the drop. This can cause downtime in production systems.
    /// Use DROP INDEX CONCURRENTLY to drop the index without blocking concurrent operations.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// DROP INDEX IF EXISTS users_email_idx;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
    /// ```
    ///
    pub RequireConcurrentIndexDeletion {
        version: "next",
        name: "requireConcurrentIndexDeletion",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("require-concurrent-index-deletion")],
    }
}

impl Rule for RequireConcurrentIndexDeletion {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::DropStmt(stmt) = &ctx.stmt() {
            if !stmt.concurrent
                && stmt.remove_type() == pgls_query::protobuf::ObjectType::ObjectIndex
            {
                diagnostics.push(RuleDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping an index non-concurrently blocks reads and writes to the table."
                    },
                ).detail(None, "Use DROP INDEX CONCURRENTLY to avoid blocking concurrent operations on the table."));
            }
        }

        diagnostics
    }
}
