//! pglinter Postgres extension integration for database linting

mod cache;
mod diagnostics;
pub mod registry;
pub mod rule;
pub mod rules;

use pgls_analyse::{AnalysisFilter, RegistryVisitor, RuleMeta};
use pgls_schema_cache::SchemaCache;
use sqlx::PgPool;

pub use cache::PglinterCache;
pub use diagnostics::{PglinterAdvices, PglinterDiagnostic};
pub use rule::PglinterRule;

/// Parameters for running pglinter
#[derive(Debug)]
pub struct PglinterParams<'a> {
    pub conn: &'a PgPool,
    pub schema_cache: &'a SchemaCache,
}

/// Visitor that collects enabled pglinter rules based on filter
struct RuleCollector<'a> {
    filter: &'a AnalysisFilter<'a>,
    enabled_rules: Vec<String>,
}

impl<'a> RegistryVisitor for RuleCollector<'a> {
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
            if let Some(code) = registry::get_rule_code(R::METADATA.name) {
                self.enabled_rules.push(code.to_string());
            }
        }
    }
}

fn collect_enabled_rules(filter: &AnalysisFilter<'_>) -> Vec<String> {
    let mut collector = RuleCollector {
        filter,
        enabled_rules: Vec::new(),
    };
    registry::visit_registry(&mut collector);
    collector.enabled_rules
}

/// Run pglinter rules against the database
pub async fn run_pglinter(
    params: PglinterParams<'_>,
    filter: &AnalysisFilter<'_>,
    cache: Option<&PglinterCache>,
) -> Result<Vec<PglinterDiagnostic>, sqlx::Error> {
    let mut results = vec![];

    // Check extension installed
    let extension_installed = cache.map(|c| c.extension_installed).unwrap_or_else(|| {
        params
            .schema_cache
            .extensions
            .iter()
            .any(|e| e.name == "pglinter")
    });

    // Collect enabled rules from config
    let enabled_rules = collect_enabled_rules(filter);

    if !extension_installed {
        if !enabled_rules.is_empty() {
            results.push(PglinterDiagnostic::extension_not_installed());
        }
        return Ok(results);
    }

    if enabled_rules.is_empty() {
        return Ok(results);
    }

    // Get disabled rules from extension
    let disabled_in_extension = match cache {
        Some(c) => c.disabled_rules.clone(),
        None => cache::get_disabled_rules(params.conn).await?,
    };

    // Check for mismatches and collect runnable rules
    let mut runnable_rules = Vec::new();
    for rule_code in &enabled_rules {
        if disabled_in_extension.contains(rule_code) {
            results.push(PglinterDiagnostic::rule_disabled_in_extension(rule_code));
        } else {
            runnable_rules.push(rule_code.clone());
        }
    }

    if runnable_rules.is_empty() {
        return Ok(results);
    }

    // Execute each rule
    for rule_code in &runnable_rules {
        if let Some(diags) = execute_rule(params.conn, rule_code).await? {
            results.extend(diags);
        }
    }

    Ok(results)
}

/// Execute a single pglinter rule using pglinter.check(rule_code)
/// Returns true if the rule detected issues
async fn execute_rule(
    conn: &PgPool,
    rule_code: &str,
) -> Result<Option<Vec<PglinterDiagnostic>>, sqlx::Error> {
    let has_issues: bool = sqlx::query_scalar("SELECT pglinter.check($1)")
        .bind(rule_code)
        .fetch_one(conn)
        .await?;

    if !has_issues {
        return Ok(None);
    }

    // Rule fired - create diagnostic from our known metadata
    if let Some(diag) = PglinterDiagnostic::from_rule_code(rule_code) {
        Ok(Some(vec![diag]))
    } else {
        Ok(None)
    }
}
