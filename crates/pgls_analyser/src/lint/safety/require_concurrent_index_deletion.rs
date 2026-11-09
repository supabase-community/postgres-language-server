use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix, fix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{DropBehavior, Token};

declare_lint_rule! {
    /// Dropping indexes non-concurrently can lock the table for reads.
    ///
    /// When dropping an index, using DROP INDEX without CONCURRENTLY will lock the table
    /// preventing reads and writes for the duration of the drop. This can cause downtime in production systems.
    /// Use DROP INDEX CONCURRENTLY to drop the index without blocking concurrent operations.
    ///
    /// The fix adds `CONCURRENTLY` where the statement surely runs outside a transaction block, and
    /// not on the indexes of partitioned tables, which Postgres can't drop concurrently. Without
    /// migration section markers, we don't know whether the migration tool runs the file in a
    /// transaction: a file with more than one statement gets no fix, and the fix is unsafe in a
    /// file with a single statement.
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
        version: "0.15.0",
        name: "requireConcurrentIndexDeletion",
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("require-concurrent-index-deletion")],
    }
}

impl LinterRule for RequireConcurrentIndexDeletion {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::DropStmt(stmt) = &ctx.stmt()
            && !stmt.concurrent
            && stmt.remove_type() == pgls_query::protobuf::ObjectType::ObjectIndex
        {
            diagnostics.push(LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping an index non-concurrently blocks reads and writes to the table."
                    },
                ).detail(None, "Use DROP INDEX CONCURRENTLY to avoid blocking concurrent operations on the table.")
                .maybe_fix(add_concurrently(ctx, stmt)));
        }

        diagnostics
    }
}

/// `DROP INDEX CONCURRENTLY` drops a single index, without `CASCADE`, outside of a transaction
/// block. It can't drop the index of a partitioned table.
fn add_concurrently(
    ctx: &LinterRuleContext<RequireConcurrentIndexDeletion>,
    stmt: &pgls_query::protobuf::DropStmt,
) -> Option<RuleFix> {
    let [object] = stmt.objects.as_slice() else {
        return None;
    };
    let Some(pgls_query::NodeEnum::List(names)) = &object.node else {
        return None;
    };
    if stmt.behavior() == DropBehavior::DropCascade
        || ctx
            .catalog()
            .is_partitioned_index(&names.items, ctx.session().search_path())
    {
        return None;
    }
    let tokens = ctx.tokens()?;
    let index = tokens.find(Token::Index)?;
    fix::add_concurrently(ctx, &tokens, index)
}
