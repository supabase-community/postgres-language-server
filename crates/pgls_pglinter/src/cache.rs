//! Pglinter extension cache for avoiding repeated database queries

use pgls_schema_cache::SchemaCache;
use rustc_hash::{FxHashMap, FxHashSet};
use sqlx::PgPool;

/// Rule message from pglinter.rule_messages table (pglinter v1.1.0+)
#[derive(Debug, Clone, Default)]
pub struct RuleMessage {
    /// Severity level (e.g., "WARNING", "ERROR")
    pub severity: String,
    /// Message template with {object} placeholder
    pub message: String,
    /// Detailed advice/description
    pub advices: String,
    /// List of fix suggestions
    pub infos: Vec<String>,
}

/// Raw JSON structure from pglinter.rule_messages
#[derive(Debug, serde::Deserialize)]
struct RuleMessageJson {
    severity: Option<String>,
    message: Option<String>,
    advices: Option<String>,
    infos: Option<Vec<String>>,
}

/// Cached pglinter extension state (loaded once, reused)
#[derive(Debug, Clone, Default)]
pub struct PglinterCache {
    /// Whether the pglinter extension is installed
    pub extension_installed: bool,
    /// Rule codes that are disabled in the pglinter extension
    pub disabled_rules: FxHashSet<String>,
    /// Rule messages from pglinter.rule_messages table (pglinter v1.1.0+)
    pub rule_messages: FxHashMap<String, RuleMessage>,
}

impl PglinterCache {
    /// Load pglinter extension state from database using official API
    pub async fn load(conn: &PgPool, schema_cache: &SchemaCache) -> Result<Self, sqlx::Error> {
        let extension_installed = schema_cache.extensions.iter().any(|e| e.name == "pglinter");

        if !extension_installed {
            return Ok(Self {
                extension_installed: false,
                disabled_rules: FxHashSet::default(),
                rule_messages: FxHashMap::default(),
            });
        }

        // Get disabled rules using pglinter.rules table
        let disabled_rules = get_disabled_rules(conn).await?;

        // Get rule messages if pglinter v1.1.0+ (rule_messages table exists)
        let rule_messages = if check_rule_messages_table_exists(conn).await? {
            fetch_rule_messages(conn).await?
        } else {
            FxHashMap::default()
        };

        Ok(Self {
            extension_installed,
            disabled_rules,
            rule_messages,
        })
    }

    /// Create initial cache from schema cache only (disabled rules will need API call later)
    pub fn from_schema_cache(schema_cache: &SchemaCache) -> Self {
        Self {
            extension_installed: schema_cache.extensions.iter().any(|e| e.name == "pglinter"),
            disabled_rules: FxHashSet::default(),
            rule_messages: FxHashMap::default(),
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

/// Check if pglinter.rule_messages table exists (requires pglinter v1.1.0+)
pub async fn check_rule_messages_table_exists(conn: &PgPool) -> Result<bool, sqlx::Error> {
    let result: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'pglinter' AND table_name = 'rule_messages'
        )",
    )
    .fetch_optional(conn)
    .await?;

    Ok(result.map(|(exists,)| exists).unwrap_or(false))
}

/// Fetch rule messages from pglinter.rule_messages table (pglinter v1.1.0+)
pub async fn fetch_rule_messages(
    conn: &PgPool,
) -> Result<FxHashMap<String, RuleMessage>, sqlx::Error> {
    let rows: Vec<(String, sqlx::types::Json<RuleMessageJson>)> =
        sqlx::query_as("SELECT code, rule_msg FROM pglinter.rule_messages")
            .fetch_all(conn)
            .await?;

    Ok(rows
        .into_iter()
        .map(|(code, json)| {
            let msg = RuleMessage {
                severity: json.severity.clone().unwrap_or_default(),
                message: json.message.clone().unwrap_or_default(),
                advices: json.advices.clone().unwrap_or_default(),
                infos: json.infos.clone().unwrap_or_default(),
            };
            (code, msg)
        })
        .collect())
}
