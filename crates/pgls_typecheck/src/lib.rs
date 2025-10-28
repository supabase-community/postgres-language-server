pub mod diagnostics;
pub mod typed_identifier;

pub use diagnostics::TypecheckDiagnostic;
use diagnostics::create_type_error;
use globset::Glob;
use itertools::Itertools;
use pgls_schema_cache::SchemaCache;
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
    // Check if the AST is not a supported statement type
    if !matches!(
        params.ast,
        pgls_query::NodeEnum::SelectStmt(_)
            | pgls_query::NodeEnum::InsertStmt(_)
            | pgls_query::NodeEnum::UpdateStmt(_)
            | pgls_query::NodeEnum::DeleteStmt(_)
            | pgls_query::NodeEnum::CommonTableExpr(_)
    ) {
        return Ok(None);
    }

    let mut conn = params.conn.acquire().await?;

    // Postgres caches prepared statements within the current DB session (connection).
    // This can cause issues if the underlying table schema changes while statements
    // are cached. By closing the connection after use, we ensure a fresh state for
    // each typecheck operation.
    conn.close_on_drop();

    let typed_replacement = apply_identifiers(
        params.identifiers,
        params.schema_cache,
        params.tree,
        params.sql,
    );

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
                params.tree,
                typed_replacement,
            )))
        }
        Err(err) => Err(err),
    }
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
