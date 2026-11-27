pub mod diagnostics;

pub use diagnostics::TypecheckDiagnostic;
use diagnostics::create_type_error;
use itertools::Itertools;
use pgls_text_size::TextSize;
use pgls_tokenizer::NamePosition;
use sqlx::pool::PoolConnection;
pub use sqlx::postgres::PgSeverity;
use sqlx::postgres::{PgDatabaseError, Postgres};
use sqlx::{Executor, PgPool};

#[derive(Debug)]
pub struct TypecheckParams<'a> {
    /// A pool without a statement cache (`statement_cache_capacity(0)`). The check prepares the
    /// statement on one of its connections and resets the session with `discard all`
    /// afterwards, which also drops what a statement cache would hold.
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pgls_query::NodeEnum,
    pub tree: &'a tree_sitter::Tree,
    /// The explicit search path of the session at this statement.
    pub search_path: &'a [String],
    /// Set when `banDuplicateColumnAssignments` already reported the statement, so Postgres'
    /// error for the same assignments, which covers the whole statement, is not reported again.
    pub duplicate_assignments_reported: bool,
}

pub async fn check_sql(
    params: TypecheckParams<'_>,
) -> Result<Option<TypecheckDiagnostic>, sqlx::Error> {
    let Some(target) = typecheck_target(params.ast, params.tree, params.sql) else {
        return Ok(None);
    };

    let mut parser = tree_sitter::Parser::new();
    let inner_tree;
    let (sql, tree, source_offset) = match target {
        TypecheckTarget::Direct => (params.sql, params.tree, TextSize::from(0)),
        TypecheckTarget::CreateQuery { sql, source_offset } => {
            #[expect(
                clippy::expect_used,
                reason = "only fails if the grammar was generated for an incompatible tree-sitter version, which the tests catch"
            )]
            parser
                .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
                .expect("Error loading sql language");
            let Some(tree) = parser.parse(sql, None) else {
                return Ok(None);
            };
            inner_tree = tree;
            (sql, &inner_tree, source_offset)
        }
    };

    let mut session = CheckSession {
        conn: params.conn.acquire().await?,
        reset: false,
    };
    let conn = &mut session.conn;

    let res = async {
        if !params.search_path.is_empty() {
            let schemas = params
                .search_path
                .iter()
                .map(|schema| NamePosition::ColId.quote(schema))
                .join(", ");
            conn.execute(&*format!("SET search_path TO {schemas};"))
                .await?;
        }
        conn.prepare(sql).await.map(drop)
    }
    .await;

    // The prepared statement and the search path stay in the session. `discard all` resets it,
    // so that the connection goes back to the pool as it came, and a prepared statement can't
    // outlive a change of the schema (#291).
    session.reset = conn.execute(sqlx::raw_sql("discard all")).await.is_ok();

    match res {
        Ok(_) => Ok(None),
        Err(sqlx::Error::Database(err)) => {
            let pg_err = err.downcast_ref::<PgDatabaseError>();
            if params.duplicate_assignments_reported && is_multiple_assignments(pg_err) {
                return Ok(None);
            }
            Ok(Some(create_type_error(pg_err, tree, sql, source_offset)))
        }
        Err(err) => Err(err),
    }
}

/// The connection of one check. It goes back to the pool only once its session was reset, and
/// closes otherwise: a check that is dropped half-way, e.g. because another check of the same
/// pull failed, would return it with the check's search path set, and the next statement a user
/// executes on the pool would run with it.
struct CheckSession {
    conn: PoolConnection<Postgres>,
    reset: bool,
}

impl Drop for CheckSession {
    fn drop(&mut self) {
        if !self.reset {
            self.conn.close_on_drop();
        }
    }
}

/// The error of [`process_matched_tle`] for a column assigned twice. The rewriter raises it after
/// parse analysis succeeded, so it hides no other error of the statement.
///
/// [`process_matched_tle`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/rewrite/rewriteHandler.c#L1081
fn is_multiple_assignments(err: &PgDatabaseError) -> bool {
    err.code() == "42601"
        && err
            .message()
            .starts_with("multiple assignments to same column")
}

enum TypecheckTarget<'a> {
    Direct,
    CreateQuery {
        sql: &'a str,
        source_offset: TextSize,
    },
}

fn typecheck_target<'a>(
    ast: &pgls_query::NodeEnum,
    tree: &tree_sitter::Tree,
    sql: &'a str,
) -> Option<TypecheckTarget<'a>> {
    match ast {
        pgls_query::NodeEnum::SelectStmt(_)
        | pgls_query::NodeEnum::InsertStmt(_)
        | pgls_query::NodeEnum::UpdateStmt(_)
        | pgls_query::NodeEnum::DeleteStmt(_)
        // Postgres can prepare MERGE since 15, the oldest version we support.
        | pgls_query::NodeEnum::MergeStmt(_)
        | pgls_query::NodeEnum::CommonTableExpr(_) => Some(TypecheckTarget::Direct),
        pgls_query::NodeEnum::CreateTableAsStmt(_) | pgls_query::NodeEnum::ViewStmt(_) => {
            let node = find_create_query(tree.root_node())?;
            if node.has_error() {
                return None;
            }
            let start = node.start_byte();
            let end = node.end_byte();
            Some(TypecheckTarget::CreateQuery {
                sql: sql.get(start..end)?,
                source_offset: TextSize::try_from(start).ok()?,
            })
        }
        _ => None,
    }
}

/// The first `create_query` node in pre-order. A loop with a cursor, since tree-sitter doesn't
/// bound the depth of its trees.
fn find_create_query(node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    let mut cursor = node.walk();
    loop {
        if cursor.node().kind() == "create_query" {
            return Some(cursor.node());
        }

        if cursor.goto_first_child() {
            continue;
        }

        // The cursor can't leave `node`: it starts there, and `goto_parent` fails at its start.
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(sql: &str) -> Option<(String, TextSize)> {
        let ast = pgls_query::parse(sql).ok()?.into_root()?;
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(sql, None)?;

        match typecheck_target(&ast, &tree, sql)? {
            TypecheckTarget::Direct => Some((sql.to_string(), TextSize::from(0))),
            TypecheckTarget::CreateQuery { sql, source_offset } => {
                Some((sql.to_string(), source_offset))
            }
        }
    }

    #[test]
    fn extracts_create_query_targets() {
        let cases = [
            (
                "CREATE TABLE t2 AS SELECT t.a FROM t",
                "SELECT t.a FROM t",
                19,
            ),
            (
                "CREATE MATERIALIZED VIEW mv1 AS SELECT t.a FROM t",
                "SELECT t.a FROM t",
                32,
            ),
            (
                "CREATE VIEW v1 AS SELECT t.a FROM t",
                "SELECT t.a FROM t",
                18,
            ),
            (
                "CREATE OR REPLACE VIEW v1 AS\n  SELECT t.a\n  FROM t",
                "SELECT t.a\n  FROM t",
                31,
            ),
        ];

        for (input, expected_sql, expected_offset) in cases {
            assert_eq!(
                target(input),
                Some((expected_sql.to_string(), TextSize::from(expected_offset)))
            );
        }
    }

    #[test]
    fn ignores_unsupported_typecheck_targets() {
        assert_eq!(target("CREATE TABLE t (id INT)"), None);
        assert_eq!(
            target("SELECT id FROM t"),
            Some(("SELECT id FROM t".to_string(), TextSize::from(0)))
        );
    }
}
