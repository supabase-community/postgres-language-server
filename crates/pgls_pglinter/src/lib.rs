//! pglinter Postgres extension integration for database linting

mod cache;
mod diagnostics;
pub mod registry;
pub mod rule;
pub mod rules;

use pgls_analyse::{AnalysisFilter, RegistryVisitor, RuleMeta};
use pgls_diagnostics::DatabaseObjectOwned;
use pgls_schema_cache::SchemaCache;
use rustc_hash::FxHashMap;
use sqlx::PgPool;

pub use cache::{PglinterCache, RuleMessage};
pub use diagnostics::{PglinterAdvices, PglinterDiagnostic};
pub use rule::PglinterRule;

/// PostgreSQL catalog OIDs for different object types
mod pg_catalog {
    pub const PG_CLASS: i64 = 1259; // tables, views, indexes, sequences
    pub const PG_PROC: i64 = 1255; // functions, procedures
    pub const PG_TYPE: i64 = 1247; // types
    pub const PG_NAMESPACE: i64 = 2615; // schemas
    pub const PG_ATTRIBUTE: i64 = 1249; // columns (objid=table oid, objsubid=column number)
}

/// A violation row returned by pglinter.get_violations()
#[derive(Debug, sqlx::FromRow)]
struct ViolationRow {
    rule_code: String,
    classid: i64,
    objid: i64,
    objsubid: i32,
}

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
        if self.filter.match_rule::<R>()
            && let Some(code) = registry::get_rule_code(R::METADATA.name)
        {
            self.enabled_rules.push(code.to_string());
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

    // Get rule messages from cache or fetch
    let rule_messages = match cache {
        Some(c) => c.rule_messages.clone(),
        None => {
            if cache::check_rule_messages_table_exists(params.conn).await? {
                cache::fetch_rule_messages(params.conn).await?
            } else {
                FxHashMap::default()
            }
        }
    };

    // Fetch all violations in one query
    let violations = fetch_violations(params.conn).await?;

    // Process violations, filtering by enabled rules and resolving objects from cache
    for violation in violations {
        // Skip violations for rules we're not checking
        if !runnable_rules.contains(&violation.rule_code) {
            continue;
        }

        // Resolve the object from the schema cache
        let db_object = resolve_object_from_cache(
            params.schema_cache,
            violation.classid,
            violation.objid,
            violation.objsubid,
        );

        // Get rule message if available
        let rule_message = rule_messages.get(&violation.rule_code);

        // Create a diagnostic for this violation
        if let Some(diag) =
            PglinterDiagnostic::from_violation(&violation.rule_code, db_object, rule_message)
        {
            results.push(diag);
        }
    }

    Ok(results)
}

/// Fetch all violations from pglinter.get_violations()
async fn fetch_violations(conn: &PgPool) -> Result<Vec<ViolationRow>, sqlx::Error> {
    sqlx::query_as::<_, ViolationRow>(
        "select rule_code, classid::bigint, objid::bigint, objsubid from pglinter.get_violations()",
    )
    .fetch_all(conn)
    .await
}

/// Resolve a Postgres object from the schema cache using its catalog OIDs
fn resolve_object_from_cache(
    schema_cache: &SchemaCache,
    classid: i64,
    objid: i64,
    objsubid: i32,
) -> Option<DatabaseObjectOwned> {
    match classid {
        pg_catalog::PG_CLASS => {
            // pg_class contains tables, views, indexes, sequences, etc.
            // Try tables first, then indexes, then sequences
            schema_cache
                .find_table_by_id(objid)
                .map(|t| DatabaseObjectOwned {
                    schema: Some(t.schema.clone()),
                    name: t.name.clone(),
                    object_type: Some(format!("{:?}", t.table_kind).to_lowercase()),
                })
                .or_else(|| {
                    schema_cache
                        .find_index_by_id(objid)
                        .map(|i| DatabaseObjectOwned {
                            schema: Some(i.schema.clone()),
                            name: i.name.clone(),
                            object_type: Some("index".to_string()),
                        })
                })
                .or_else(|| {
                    schema_cache
                        .find_sequence_by_id(objid)
                        .map(|s| DatabaseObjectOwned {
                            schema: Some(s.schema.clone()),
                            name: s.name.clone(),
                            object_type: Some("sequence".to_string()),
                        })
                })
        }
        pg_catalog::PG_PROC => {
            // Functions and procedures
            schema_cache
                .find_function_by_id(objid)
                .map(|f| DatabaseObjectOwned {
                    schema: Some(f.schema.clone()),
                    name: f.name.clone(),
                    object_type: Some(format!("{:?}", f.kind).to_lowercase()),
                })
        }
        pg_catalog::PG_TYPE => {
            // Types
            schema_cache
                .find_type_by_id(objid)
                .map(|t| DatabaseObjectOwned {
                    schema: Some(t.schema.clone()),
                    name: t.name.clone(),
                    object_type: Some("type".to_string()),
                })
        }
        pg_catalog::PG_NAMESPACE => {
            // Schemas
            schema_cache
                .find_schema_by_id(objid)
                .map(|s| DatabaseObjectOwned {
                    schema: None,
                    name: s.name.clone(),
                    object_type: Some("schema".to_string()),
                })
        }
        pg_catalog::PG_ATTRIBUTE => {
            // Columns: objid is table OID, objsubid is column number (attnum)
            // Find the column by table OID and column number
            let col_num = i64::from(objsubid);
            schema_cache
                .columns
                .iter()
                .find(|c| c.table_oid == objid && c.number == col_num)
                .map(|c| DatabaseObjectOwned {
                    schema: Some(c.schema_name.clone()),
                    name: format!("{}.{}", c.table_name, c.name),
                    object_type: Some("column".to_string()),
                })
        }
        _ => {
            // Unknown catalog - we can't resolve this object from the cache
            None
        }
    }
}
