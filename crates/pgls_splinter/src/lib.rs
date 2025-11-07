mod convert;
mod diagnostics;
mod query;

use sqlx::PgPool;

pub use diagnostics::{SplinterAdvices, SplinterDiagnostic};
pub use query::SplinterQueryResult;

#[derive(Debug)]
pub struct SplinterParams<'a> {
    pub conn: &'a PgPool,
}

async fn check_required_roles(conn: &PgPool) -> Result<bool, sqlx::Error> {
    let required_roles = ["anon", "authenticated", "service_role"];

    let existing_roles: Vec<String> =
        sqlx::query_scalar("SELECT rolname FROM pg_roles WHERE rolname = ANY($1)")
            .bind(&required_roles[..])
            .fetch_all(conn)
            .await?;

    // Check if all required roles exist
    let all_exist = required_roles
        .iter()
        .all(|role| existing_roles.contains(&(*role).to_string()));

    Ok(all_exist)
}

pub async fn run_splinter(
    params: SplinterParams<'_>,
) -> Result<Vec<SplinterDiagnostic>, sqlx::Error> {
    // check if required supabase roles exist
    // if they don't exist, return empty diagnostics since splinter is supabase-specific
    // opened an issue to make it less supabase-specific: https://github.com/supabase/splinter/issues/135
    let has_roles = check_required_roles(params.conn).await?;
    if !has_roles {
        return Ok(Vec::new());
    }

    let results = query::load_splinter_results(params.conn).await?;

    let diagnostics: Vec<SplinterDiagnostic> = results.into_iter().map(Into::into).collect();

    Ok(diagnostics)
}
