mod diagnostics;
mod typed_identifier;

pub use diagnostics::TypecheckDiagnostic;
use diagnostics::create_type_error;
use pgt_text_size::TextRange;
use sqlx::postgres::PgDatabaseError;
pub use sqlx::postgres::PgSeverity;
use sqlx::{Executor, PgPool};
use typed_identifier::apply_identifiers;
pub use typed_identifier::{IdentifierType, TypedIdentifier};

#[derive(Debug)]
pub struct TypecheckParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pgt_query::NodeEnum,
    pub tree: &'a tree_sitter::Tree,
    pub schema_cache: &'a pgt_schema_cache::SchemaCache,
    pub identifiers: Vec<TypedIdentifier>,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub code: String,
    pub severity: PgSeverity,
    pub position: Option<usize>,
    pub range: Option<TextRange>,
    pub table: Option<String>,
    pub column: Option<String>,
    pub data_type: Option<String>,
    pub constraint: Option<String>,
}

pub async fn check_sql(
    params: TypecheckParams<'_>,
) -> Result<Option<TypecheckDiagnostic>, sqlx::Error> {
    // Check if the AST is not a supported statement type
    if !matches!(
        params.ast,
        pgt_query::NodeEnum::SelectStmt(_)
            | pgt_query::NodeEnum::InsertStmt(_)
            | pgt_query::NodeEnum::UpdateStmt(_)
            | pgt_query::NodeEnum::DeleteStmt(_)
            | pgt_query::NodeEnum::CommonTableExpr(_)
    ) {
        return Ok(None);
    }

    let mut conn = params.conn.acquire().await?;

    // Postgres caches prepared statements within the current DB session (connection).
    // This can cause issues if the underlying table schema changes while statements
    // are cached. By closing the connection after use, we ensure a fresh state for
    // each typecheck operation.
    conn.close_on_drop();

    let (prepared, positions_valid) = apply_identifiers(
        params.identifiers,
        params.schema_cache,
        params.tree,
        params.sql,
    );

    let res = conn.prepare(&prepared).await;

    match res {
        Ok(_) => Ok(None),
        Err(sqlx::Error::Database(err)) => {
            let pg_err = err.downcast_ref::<PgDatabaseError>();
            Ok(Some(create_type_error(
                pg_err,
                params.tree,
                positions_valid,
            )))
        }
        Err(err) => Err(err),
    }
}
