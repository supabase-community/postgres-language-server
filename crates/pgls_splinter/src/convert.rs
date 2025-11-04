use pgls_diagnostics::{Category, Severity, category};
use serde_json::Value;

use crate::{SplinterAdvices, SplinterDiagnostic, SplinterQueryResult};

impl From<SplinterQueryResult> for SplinterDiagnostic {
    fn from(result: SplinterQueryResult) -> Self {
        let severity = parse_severity(&result.level);

        // Extract common fields from metadata
        let (schema, object_name, object_type, additional_metadata) =
            extract_metadata_fields(&result.metadata);

        // for now, we just take the first category as the group
        let group = result
            .categories
            .first()
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        SplinterDiagnostic {
            category: rule_name_to_category(&result.name, &group),
            message: result.detail.into(),
            severity,
            advices: SplinterAdvices {
                description: result.description,
                schema,
                object_name,
                object_type,
                remediation_url: build_remediation_url(&result.name),
                additional_metadata,
            },
        }
    }
}

/// Build remediation URL from rule name
/// Maps rule names to their Supabase linter documentation
fn build_remediation_url(name: &str) -> String {
    // Map rule names to their lint IDs
    let lint_id = match name {
        "unindexed_foreign_keys" => "0001_unindexed_foreign_keys",
        "auth_users_exposed" => "0002_auth_users_exposed",
        "auth_rls_initplan" => "0003_auth_rls_initplan",
        "no_primary_key" => "0004_no_primary_key",
        "unused_index" => "0005_unused_index",
        "multiple_permissive_policies" => "0006_multiple_permissive_policies",
        "policy_exists_rls_disabled" => "0007_policy_exists_rls_disabled",
        "rls_enabled_no_policy" => "0008_rls_enabled_no_policy",
        "duplicate_index" => "0009_duplicate_index",
        "security_definer_view" => "0010_security_definer_view",
        "function_search_path_mutable" => "0011_function_search_path_mutable",
        "rls_disabled_in_public" => "0013_rls_disabled_in_public",
        "extension_in_public" => "0014_extension_in_public",
        "rls_references_user_metadata" => "0015_rls_references_user_metadata",
        "materialized_view_in_api" => "0016_materialized_view_in_api",
        "foreign_table_in_api" => "0017_foreign_table_in_api",
        "unsupported_reg_types" => "unsupported_reg_types",
        "insecure_queue_exposed_in_api" => "0019_insecure_queue_exposed_in_api",
        "table_bloat" => "0020_table_bloat",
        "fkey_to_auth_unique" => "0021_fkey_to_auth_unique",
        "extension_versions_outdated" => "0022_extension_versions_outdated",
        _ => return "https://supabase.com/docs/guides/database/database-linter".to_string(),
    };

    format!("https://supabase.com/docs/guides/database/database-linter?lint={lint_id}")
}

/// Parse severity level from the query result
fn parse_severity(level: &str) -> Severity {
    match level {
        "INFO" => Severity::Information,
        "WARN" => Severity::Warning,
        "ERROR" => Severity::Error,
        _ => Severity::Information, // default to info
    }
}

/// Convert rule name and group to a Category
/// Note: Rule names use snake_case, but categories use camelCase
fn rule_name_to_category(name: &str, group: &str) -> &'static Category {
    match (group, name) {
        ("performance", "unindexed_foreign_keys") => {
            category!("splinter/performance/unindexedForeignKeys")
        }
        ("performance", "auth_rls_initplan") => {
            category!("splinter/performance/authRlsInitplan")
        }
        ("performance", "no_primary_key") => category!("splinter/performance/noPrimaryKey"),
        ("performance", "unused_index") => category!("splinter/performance/unusedIndex"),
        ("performance", "duplicate_index") => category!("splinter/performance/duplicateIndex"),
        ("performance", "table_bloat") => category!("splinter/performance/tableBloat"),
        ("performance", "multiple_permissive_policies") => {
            category!("splinter/performance/multiplePermissivePolicies")
        }
        ("security", "auth_users_exposed") => category!("splinter/security/authUsersExposed"),
        ("security", "extension_versions_outdated") => {
            category!("splinter/security/extensionVersionsOutdated")
        }
        ("security", "policy_exists_rls_disabled") => {
            category!("splinter/security/policyExistsRlsDisabled")
        }
        ("security", "rls_enabled_no_policy") => {
            category!("splinter/security/rlsEnabledNoPolicy")
        }
        ("security", "security_definer_view") => {
            category!("splinter/security/securityDefinerView")
        }
        ("security", "function_search_path_mutable") => {
            category!("splinter/security/functionSearchPathMutable")
        }
        ("security", "rls_disabled_in_public") => {
            category!("splinter/security/rlsDisabledInPublic")
        }
        ("security", "extension_in_public") => category!("splinter/security/extensionInPublic"),
        ("security", "rls_references_user_metadata") => {
            category!("splinter/security/rlsReferencesUserMetadata")
        }
        ("security", "materialized_view_in_api") => {
            category!("splinter/security/materializedViewInApi")
        }
        ("security", "foreign_table_in_api") => {
            category!("splinter/security/foreignTableInApi")
        }
        ("security", "unsupported_reg_types") => {
            category!("splinter/security/unsupportedRegTypes")
        }
        ("security", "insecure_queue_exposed_in_api") => {
            category!("splinter/security/insecureQueueExposedInApi")
        }
        ("security", "fkey_to_auth_unique") => category!("splinter/security/fkeyToAuthUnique"),
        _ => category!("splinter/unknown/unknown"),
    }
}

/// Extract common metadata fields and return the rest as additional_metadata
fn extract_metadata_fields(
    metadata: &Value,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<Value>,
) {
    if let Some(obj) = metadata.as_object() {
        let schema = obj.get("schema").and_then(|v| v.as_str()).map(String::from);

        let object_name = obj.get("name").and_then(|v| v.as_str()).map(String::from);

        let object_type = obj.get("type").and_then(|v| v.as_str()).map(String::from);

        // Create a new object without the common fields
        let mut additional = obj.clone();
        additional.remove("schema");
        additional.remove("name");
        additional.remove("type");

        let additional_metadata = if additional.is_empty() {
            None
        } else {
            Some(Value::Object(additional))
        };

        (schema, object_name, object_type, additional_metadata)
    } else {
        (None, None, None, Some(metadata.clone()))
    }
}
