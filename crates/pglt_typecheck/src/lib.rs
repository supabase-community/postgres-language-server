mod diagnostics;

use diagnostics::create_type_error;
pub use diagnostics::TypecheckDiagnostic;
use sqlx::postgres::PgDatabaseError;
pub use sqlx::postgres::PgSeverity;
use sqlx::Executor;
use sqlx::PgPool;
use text_size::TextRange;

#[derive(Debug)]
pub struct TypecheckParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pglt_query_ext::NodeEnum,
    pub tree: Option<&'a tree_sitter::Tree>,
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

pub async fn check_sql(params: TypecheckParams<'_>) -> Option<TypecheckDiagnostic> {
    // Check if the AST is not a supported statement type
    if !matches!(
        params.ast,
        pglt_query_ext::NodeEnum::SelectStmt(_)
            | pglt_query_ext::NodeEnum::InsertStmt(_)
            | pglt_query_ext::NodeEnum::UpdateStmt(_)
            | pglt_query_ext::NodeEnum::DeleteStmt(_)
            | pglt_query_ext::NodeEnum::CommonTableExpr(_)
    ) {
        return None;
    }

    let res = params.conn.prepare(params.sql).await;

    match res {
        Ok(_) => None,
        Err(sqlx::Error::Database(err)) => {
            let pg_err = err.downcast_ref::<PgDatabaseError>();
            Some(create_type_error(pg_err, params.tree))
        }
        Err(_) => None,
    }
}
