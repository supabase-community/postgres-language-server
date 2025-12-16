mod convert;
mod diagnostics;
mod query;
pub mod registry;
pub mod rule;
pub mod rules;

use sqlx::PgPool;

pub use diagnostics::{SplinterAdvices, SplinterDiagnostic};
pub use query::SplinterQueryResult;
pub use rule::SplinterRule;

#[derive(Debug)]
pub struct SplinterParams<'a> {
    pub conn: &'a PgPool,
}

async fn check_supabase_roles(conn: &PgPool) -> Result<bool, sqlx::Error> {
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
    let mut all_results = Vec::new();

    let generic_results = query::load_generic_splinter_results(params.conn).await?;
    all_results.extend(generic_results);

    // Only run Supabase-specific rules if the required roles exist
    let has_supabase_roles = check_supabase_roles(params.conn).await?;
    if has_supabase_roles {
        let supabase_results = query::load_supabase_splinter_results(params.conn).await?;
        all_results.extend(supabase_results);
    }

    let diagnostics: Vec<SplinterDiagnostic> = all_results.into_iter().map(Into::into).collect();

    Ok(diagnostics)
}
