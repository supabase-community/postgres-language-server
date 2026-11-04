use crate::{
    LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix, fix,
    linter_context::is_reindex_concurrent,
};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// `REINDEX` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock on the table.
    ///
    /// This blocks all reads and writes until the reindex completes. Use `REINDEX CONCURRENTLY`
    /// to rebuild the index without blocking concurrent operations.
    ///
    /// The fix adds `CONCURRENTLY` where the statement surely runs outside a transaction block, and
    /// not on system catalogs, which Postgres can't reindex concurrently. Without migration section
    /// markers, we don't know whether the migration tool runs the file in a transaction: a file
    /// with more than one statement gets no fix, and the fix is unsafe in a file with a single
    /// statement.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// reindex index my_index;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// reindex index concurrently my_index;
    /// ```
    ///
    pub RequireConcurrentReindex {
        version: "0.25.0",
        name: "requireConcurrentReindex",
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Pgfence("reindex-non-concurrent")],
    }
}

impl LinterRule for RequireConcurrentReindex {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::ReindexStmt(stmt) = &ctx.stmt()
            && !is_reindex_concurrent(stmt)
        {
            diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            <Emphasis>"REINDEX"</Emphasis>" without "<Emphasis>"CONCURRENTLY"</Emphasis>" blocks all table access."
                        },
                    )
                    .detail(
                        None,
                        "Use REINDEX CONCURRENTLY to rebuild the index without blocking reads and writes.",
                    )
                    .maybe_fix(add_concurrently(ctx, stmt)),
                );
        }

        diagnostics
    }
}

/// `REINDEX CONCURRENTLY` can't run in a transaction block, and not on system catalogs. An
/// explicit `CONCURRENTLY` option in the parentheses is left alone.
fn add_concurrently(
    ctx: &LinterRuleContext<RequireConcurrentReindex>,
    stmt: &pgls_query::protobuf::ReindexStmt,
) -> Option<RuleFix> {
    use pgls_query::protobuf::Token;

    let has_concurrently_option = stmt.params.iter().any(|param| {
        matches!(&param.node, Some(pgls_query::NodeEnum::DefElem(def)) if def.defname.eq_ignore_ascii_case("concurrently"))
    });
    if has_concurrently_option || is_system_catalog(stmt) {
        return None;
    }
    let tokens = ctx.tokens()?;
    let kind = tokens.find_any(&[Token::Index, Token::Table, Token::Schema, Token::Database])?;
    fix::add_concurrently(ctx, &tokens, kind)
}

/// Whether the statement reindexes system catalogs, which fails with `CONCURRENTLY`. With
/// `CONCURRENTLY`, `REINDEX SCHEMA pg_catalog` skips all of them. Unqualified names that start
/// with `pg_` likely name a system catalog, since `pg_catalog` comes first on the search path.
fn is_system_catalog(stmt: &pgls_query::protobuf::ReindexStmt) -> bool {
    use pgls_query::protobuf::ReindexObjectType;

    let is_system_schema = |schema: &str| matches!(schema, "pg_catalog" | "pg_toast");
    match stmt.kind() {
        ReindexObjectType::ReindexObjectSystem => true,
        ReindexObjectType::ReindexObjectSchema => is_system_schema(&stmt.name),
        ReindexObjectType::ReindexObjectIndex | ReindexObjectType::ReindexObjectTable => {
            stmt.relation.as_ref().is_some_and(|relation| {
                if relation.schemaname.is_empty() {
                    relation.relname.starts_with("pg_")
                } else {
                    is_system_schema(&relation.schemaname)
                }
            })
        }
        _ => false,
    }
}
