//! What the pglinter extension's tables say: the rules it disables and its rule messages.

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

/// Get disabled rules by querying the pglinter.rules table
/// Uses the rules table directly since show_rules() only outputs to NOTICE
pub(crate) async fn get_disabled_rules(conn: &PgPool) -> Result<FxHashSet<String>, sqlx::Error> {
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
pub(crate) async fn check_rule_messages_table_exists(conn: &PgPool) -> Result<bool, sqlx::Error> {
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
pub(crate) async fn fetch_rule_messages(
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
