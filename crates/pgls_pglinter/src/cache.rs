//! Pglinter extension cache for avoiding repeated database queries

use pgls_schema_cache::SchemaCache;
use rustc_hash::FxHashSet;
use sqlx::PgPool;

/// Cached pglinter extension state (loaded once, reused)
#[derive(Debug, Clone, Default)]
pub struct PglinterCache {
    /// Whether the pglinter extension is installed
    pub extension_installed: bool,
    /// Rule codes that are disabled in the pglinter extension
    pub disabled_rules: FxHashSet<String>,
}

impl PglinterCache {
    /// Load pglinter extension state from database using official API
    pub async fn load(conn: &PgPool, schema_cache: &SchemaCache) -> Result<Self, sqlx::Error> {
        let extension_installed = schema_cache.extensions.iter().any(|e| e.name == "pglinter");

        if !extension_installed {
            return Ok(Self {
                extension_installed: false,
                disabled_rules: FxHashSet::default(),
            });
        }

        // Get disabled rules using pglinter.show_rules() - single query
        let disabled_rules = get_disabled_rules(conn).await?;

        Ok(Self {
            extension_installed,
            disabled_rules,
        })
    }

    /// Create initial cache from schema cache only (disabled rules will need API call later)
    pub fn from_schema_cache(schema_cache: &SchemaCache) -> Self {
        Self {
            extension_installed: schema_cache.extensions.iter().any(|e| e.name == "pglinter"),
            disabled_rules: FxHashSet::default(),
        }
    }
}

/// Get disabled rules by querying the pglinter.rules table
/// Uses the rules table directly since show_rules() only outputs to NOTICE
pub async fn get_disabled_rules(conn: &PgPool) -> Result<FxHashSet<String>, sqlx::Error> {
    let rows: Vec<(String, bool)> = sqlx::query_as("SELECT code, enable FROM pglinter.rules")
        .fetch_all(conn)
        .await?;

    Ok(rows
        .into_iter()
        .filter(|(_, enabled)| !enabled)
        .map(|(code, _)| code)
        .collect())
}
