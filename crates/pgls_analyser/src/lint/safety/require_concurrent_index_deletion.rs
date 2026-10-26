use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix, fix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_catalog::{CatalogView, RelationKind};
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
    if stmt.objects.len() != 1
        || stmt.behavior() == DropBehavior::DropCascade
        || is_partitioned_index(ctx, &stmt.objects[0])
    {
        return None;
    }
    let tokens = ctx.tokens()?;
    let index = tokens.find(Token::Index)?;
    fix::add_concurrently(ctx, &tokens, index)
}

/// Whether the index is on a partitioned table, as far as we know: the index is created before
/// in the migration or it is in the database.
fn is_partitioned_index(
    ctx: &LinterRuleContext<RequireConcurrentIndexDeletion>,
    object: &pgls_query::protobuf::Node,
) -> bool {
    let Some(pgls_query::NodeEnum::List(list)) = &object.node else {
        return false;
    };
    let names: Vec<&str> = list
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Some(pgls_query::NodeEnum::String(name)) => Some(name.sval.as_str()),
            _ => None,
        })
        .collect();
    let (schema, name) = match names.as_slice() {
        [name] => (None, *name),
        [.., schema, name] => (Some(*schema), *name),
        [] => return false,
    };

    // An index is in the schema of its table. Indexes created before in the migration match by
    // name only.
    let created = ctx
        .file_context()
        .previous_stmts()
        .iter()
        .rev()
        .find_map(|stmt| match stmt {
            pgls_query::NodeEnum::IndexStmt(index) if index.idxname == name => {
                let relation = index.relation.as_ref()?;
                let schema = Some(relation.schemaname.as_str()).filter(|schema| !schema.is_empty());
                Some((schema, relation.relname.as_str()))
            }
            _ => None,
        });
    let table = created.or_else(|| {
        ctx.snapshot()?
            .indexes
            .iter()
            .find(|index| index.name == name && schema.is_none_or(|schema| index.schema == schema))
            .map(|index| (Some(index.schema.as_str()), index.table_name.as_str()))
    });
    table.is_some_and(|(schema, table)| {
        ctx.catalog()
            .relation(schema, table, ctx.session().search_path())
            .found()
            .is_some_and(|relation| relation.kind == RelationKind::PartitionedTable)
    })
}
