use pgls_diagnostics::{Category, DatabaseObjectOwned, Severity, category};
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
            db_object: object_name.as_ref().map(|name| DatabaseObjectOwned {
                schema: schema.clone(),
                name: name.clone(),
                object_type: object_type.clone(),
            }),
            advices: SplinterAdvices {
                description: result.description,
                schema,
                object_name,
                object_type,
                remediation: result.remediation,
                additional_metadata,
            },
        }
    }
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
    // we cannot use convert_case here because category! macro requires a string literal
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
        _ => panic!("Unknown splinter rule: {group}/{name}"),
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
