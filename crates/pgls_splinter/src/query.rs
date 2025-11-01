use serde_json::Value;
use sqlx::PgPool;

/// Raw query result from the Splinter SQL query.
/// This struct represents a single linting issue found in the database.
#[derive(Debug)]
pub struct SplinterQueryResult {
    /// Unique identifier for the lint rule (e.g., "unindexed_foreign_keys")
    pub name: String,

    /// Human-readable title for the rule (e.g., "Unindexed foreign keys")
    pub title: String,

    /// Severity level: "INFO", "WARN", or "ERROR"
    pub level: String,

    /// Facing: "EXTERNAL" or "INTERNAL"
    pub facing: String,

    /// Categories this issue belongs to (e.g., ["PERFORMANCE"], ["SECURITY"])
    pub categories: Vec<String>,

    /// General description of what this rule detects
    pub description: String,

    /// Specific detail about this particular violation
    pub detail: String,

    /// URL to documentation/remediation guide
    pub remediation: String,

    /// Structured metadata about the database objects involved
    /// Contains common keys: schema, name, type
    /// Plus rule-specific fields like fkey_name, column, indexes, etc.
    pub metadata: Value,

    /// Unique cache key for this specific issue
    pub cache_key: String,
}

pub async fn load_splinter_results(pool: &PgPool) -> Result<Vec<SplinterQueryResult>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // this is done by the splinter.sql file normally, but we remove it so that sqlx can work with
    // the file properly.
    sqlx::query("set local search_path = ''")
        .execute(&mut *tx)
        .await?;

    let results = sqlx::query_file_as!(SplinterQueryResult, "vendor/splinter.sql")
        .fetch_all(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(results)
}
