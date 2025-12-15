mod convert;
mod diagnostics;
mod query;
pub mod registry;
pub mod rule;
pub mod rules;

use pgls_analyse::{AnalysisFilter, RegistryVisitor, RuleMeta};
use pgls_schema_cache::SchemaCache;
use sqlx::PgPool;

pub use diagnostics::{SplinterAdvices, SplinterDiagnostic};
pub use query::SplinterQueryResult;
pub use rule::SplinterRule;

#[derive(Debug)]
pub struct SplinterParams<'a> {
    pub conn: &'a PgPool,
    pub schema_cache: Option<&'a SchemaCache>,
}

/// Visitor that collects enabled splinter rules based on filter
struct SplinterRuleCollector<'a> {
    filter: &'a AnalysisFilter<'a>,
    enabled_rules: Vec<String>, // rule names in camelCase
}

impl<'a> RegistryVisitor for SplinterRuleCollector<'a> {
    fn record_category<C: pgls_analyse::GroupCategory>(&mut self) {
        if self.filter.match_category::<C>() {
            C::record_groups(self);
        }
    }

    fn record_group<G: pgls_analyse::RuleGroup>(&mut self) {
        if self.filter.match_group::<G>() {
            G::record_rules(self);
        }
    }

    fn record_rule<R: RuleMeta>(&mut self) {
        if self.filter.match_rule::<R>() {
            self.enabled_rules.push(R::METADATA.name.to_string());
        }
    }
}

pub async fn run_splinter(
    params: SplinterParams<'_>,
    filter: &AnalysisFilter<'_>,
) -> Result<Vec<SplinterDiagnostic>, sqlx::Error> {
    // Use visitor pattern to collect enabled rules
    let mut collector = SplinterRuleCollector {
        filter,
        enabled_rules: Vec::new(),
    };
    crate::registry::visit_registry(&mut collector);

    // If no rules are enabled, return early
    if collector.enabled_rules.is_empty() {
        return Ok(Vec::new());
    }

    // Check if Supabase roles exist (anon, authenticated, service_role)
    let has_supabase_roles = params.schema_cache.is_some_and(|cache| {
        let required_roles = ["anon", "authenticated", "service_role"];
        required_roles.iter().all(|role_name| {
            cache
                .roles
                .iter()
                .any(|role| role.name.as_str() == *role_name)
        })
    });

    // Build dynamic SQL query from enabled rules
    // Filter out Supabase-specific rules if Supabase roles don't exist
    // SQL content is embedded at compile time using include_str! for performance
    let mut sql_queries = Vec::new();

    for rule_name in &collector.enabled_rules {
        // Skip Supabase-specific rules if Supabase roles don't exist
        if !has_supabase_roles && crate::registry::rule_requires_supabase(rule_name) {
            continue;
        }

        // Get embedded SQL content (compile-time included)
        if let Some(sql) = crate::registry::get_sql_content(rule_name) {
            sql_queries.push(sql);
        }
    }

    // If no SQL files could be read, return early
    if sql_queries.is_empty() {
        return Ok(Vec::new());
    }

    // Combine SQL queries with UNION ALL
    // Some SQL files are wrapped in parentheses, some are not
    // Ensure all queries are wrapped for valid UNION ALL syntax
    let processed_queries: Vec<String> = sql_queries
        .iter()
        .map(|sql| {
            let trimmed = sql.trim();
            // Wrap in parentheses if not already wrapped
            if trimmed.starts_with('(') && trimmed.ends_with(')') {
                trimmed.to_string()
            } else {
                format!("({trimmed})")
            }
        })
        .collect();
    // Add ORDER BY to ensure deterministic ordering across all results
    let combined_sql = format!(
        "SELECT * FROM (\n{}\n) AS all_results ORDER BY \"cache_key!\"",
        processed_queries.join("\n\nUNION ALL\n\n")
    );

    // Execute the combined query
    let mut tx = params.conn.begin().await?;

    // Set search path as done in the original implementation
    sqlx::query("set local search_path = ''")
        .execute(&mut *tx)
        .await?;

    let results = sqlx::query_as::<_, SplinterQueryResult>(&combined_sql)
        .fetch_all(&mut *tx)
        .await?;

    tx.commit().await?;

    // Convert results to diagnostics
    let diagnostics: Vec<SplinterDiagnostic> = results.into_iter().map(Into::into).collect();

    Ok(diagnostics)
}
