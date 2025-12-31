use serde_json::Value;
use sqlx::Row;

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

// Implement FromRow manually since we're using dynamic SQL
// Column names include "!" suffix (e.g., "name!") which indicates NOT NULL in SQL files
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for SplinterQueryResult {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SplinterQueryResult {
            name: row.try_get("name!")?,
            title: row.try_get("title!")?,
            level: row.try_get("level!")?,
            facing: row.try_get("facing!")?,
            categories: row.try_get("categories!")?,
            description: row.try_get("description!")?,
            detail: row.try_get("detail!")?,
            remediation: row.try_get("remediation!")?,
            metadata: row.try_get("metadata!")?,
            cache_key: row.try_get("cache_key!")?,
        })
    }
}
