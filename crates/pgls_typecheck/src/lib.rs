pub mod diagnostics;
pub mod typed_identifier;

pub use diagnostics::TypecheckDiagnostic;
use diagnostics::create_type_error;
use globset::Glob;
use itertools::Itertools;
use pgls_schema_cache::SchemaCache;
use pgls_text_size::TextSize;
use sqlx::postgres::PgDatabaseError;
pub use sqlx::postgres::PgSeverity;
use sqlx::{Executor, PgPool};
use typed_identifier::apply_identifiers;
pub use typed_identifier::{IdentifierReplacement, IdentifierType, TypedIdentifier};

#[derive(Debug)]
pub struct TypecheckParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pgls_query::NodeEnum,
    pub tree: &'a tree_sitter::Tree,
    pub schema_cache: &'a pgls_schema_cache::SchemaCache,
    pub identifiers: Vec<TypedIdentifier>,
    /// Set of glob patterns that will be matched against the schemas in the database.
    /// Each matching schema will be added to the search_path for the typecheck.
    pub search_path_patterns: Vec<String>,
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

    let mut conn = params.conn.acquire().await?;

    // Postgres caches prepared statements within the current DB session (connection).
    // This can cause issues if the underlying table schema changes while statements
    // are cached. By closing the connection after use, we ensure a fresh state for
    // each typecheck operation.
    conn.close_on_drop();

    let typed_replacement = apply_identifiers(params.identifiers, params.schema_cache, tree, sql);

    let mut search_path_schemas =
        get_schemas_in_search_path(params.schema_cache, params.search_path_patterns);

    if !search_path_schemas.is_empty() {
        // Always include public if we have any schemas in search path
        if !search_path_schemas.contains(&"public") {
            search_path_schemas.push("public");
        }

        let search_path_query = format!("SET search_path TO {};", search_path_schemas.join(", "));
        conn.execute(&*search_path_query).await?;
    }

    let res = conn
        .prepare(typed_replacement.text_replacement().text())
        .await;

    match res {
        Ok(_) => Ok(None),
        Err(sqlx::Error::Database(err)) => {
            let pg_err = err.downcast_ref::<PgDatabaseError>();
            Ok(Some(create_type_error(
                pg_err,
                tree,
                typed_replacement,
                source_offset,
            )))
        }
        Err(err) => Err(err),
    }
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

fn find_create_query(node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    if node.kind() == "create_query" {
        return Some(node);
    }

    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(find_create_query)
}

fn get_schemas_in_search_path(schema_cache: &SchemaCache, glob_patterns: Vec<String>) -> Vec<&str> {
    // iterate over glob_patterns on the outside to keep the order
    glob_patterns
        .iter()
        .filter_map(|pattern| {
            if let Ok(glob) = Glob::new(pattern) {
                let matcher = glob.compile_matcher();

                Some(
                    schema_cache
                        .schemas
                        .iter()
                        .filter_map(|s| {
                            if matcher.is_match(s.name.as_str()) {
                                Some(s.name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&str>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .unique()
        .collect()
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
