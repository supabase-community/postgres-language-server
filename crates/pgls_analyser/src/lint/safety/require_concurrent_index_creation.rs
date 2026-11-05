use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix, fix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_catalog::{CatalogView, RelationKind};
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
    /// The fix adds `CONCURRENTLY` where the statement surely runs outside a transaction block, and
    /// not on partitioned tables, which Postgres can't index concurrently. Without migration
    /// section markers, we don't know whether the migration tool runs the file in a transaction: a
    /// file with more than one statement gets no fix, and the fix is unsafe in a file with a single
    /// statement.
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

        // An index on a table created earlier in the transaction doesn't block anyone.
        let Some(relation) = &stmt.relation else {
            return diagnostics;
        };
        if ctx
            .session()
            .has_created_object(&relation.schemaname, &relation.relname)
        {
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
            .maybe_fix(add_concurrently(ctx, stmt)),
        );

        diagnostics
    }
}

/// `CREATE INDEX CONCURRENTLY` can't run in a transaction block, and not on a partitioned
/// table.
fn add_concurrently(
    ctx: &LinterRuleContext<RequireConcurrentIndexCreation>,
    stmt: &pgls_query::protobuf::IndexStmt,
) -> Option<RuleFix> {
    let relation = stmt.relation.as_ref()?;
    let schema = Some(relation.schemaname.as_str()).filter(|schema| !schema.is_empty());
    let is_partitioned = ctx
        .catalog()
        .relation(schema, &relation.relname, ctx.session().search_path())
        .found()
        .is_some_and(|relation| relation.kind == RelationKind::PartitionedTable);
    if is_partitioned {
        return None;
    }
    let tokens = ctx.tokens()?;
    let index = tokens.find(Token::Index)?;
    fix::add_concurrently(ctx, &tokens, index)
}
