use pgls_diagnostics::{Category, Severity, category};
use serde_json::Value;

use crate::{SplinterAdvices, SplinterDiagnostic, SplinterQueryResult};

impl From<SplinterQueryResult> for SplinterDiagnostic {
    fn from(result: SplinterQueryResult) -> Self {
        let severity = parse_severity(&result.level);

        // Extract common fields from metadata
        let (schema, object_name, object_type, additional_metadata) =
            extract_metadata_fields(&result.metadata);

        SplinterDiagnostic {
            category: rule_name_to_category(&result.name),
            message: result.detail.into(),
            severity,
            advices: SplinterAdvices {
                description: result.description,
                schema,
                object_name,
                object_type,
                remediation_url: result.remediation,
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

/// Convert rule name to a Category
/// Note: Rule names use snake_case, but categories use camelCase
fn rule_name_to_category(name: &str) -> &'static Category {
    match name {
        "unindexed_foreign_keys" => category!("dblint/splinter/unindexedForeignKeys"),
        "auth_users_exposed" => category!("dblint/splinter/authUsersExposed"),
        "auth_rls_initplan" => category!("dblint/splinter/authRlsInitplan"),
        "no_primary_key" => category!("dblint/splinter/noPrimaryKey"),
        "unused_index" => category!("dblint/splinter/unusedIndex"),
        "multiple_permissive_policies" => category!("dblint/splinter/multiplePermissivePolicies"),
        "policy_exists_rls_disabled" => category!("dblint/splinter/policyExistsRlsDisabled"),
        "rls_enabled_no_policy" => category!("dblint/splinter/rlsEnabledNoPolicy"),
        "duplicate_index" => category!("dblint/splinter/duplicateIndex"),
        "security_definer_view" => category!("dblint/splinter/securityDefinerView"),
        "function_search_path_mutable" => category!("dblint/splinter/functionSearchPathMutable"),
        "rls_disabled_in_public" => category!("dblint/splinter/rlsDisabledInPublic"),
        "extension_in_public" => category!("dblint/splinter/extensionInPublic"),
        "rls_references_user_metadata" => category!("dblint/splinter/rlsReferencesUserMetadata"),
        "materialized_view_in_api" => category!("dblint/splinter/materializedViewInApi"),
        "foreign_table_in_api" => category!("dblint/splinter/foreignTableInApi"),
        "unsupported_reg_types" => category!("dblint/splinter/unsupportedRegTypes"),
        "insecure_queue_exposed_in_api" => category!("dblint/splinter/insecureQueueExposedInApi"),
        "table_bloat" => category!("dblint/splinter/tableBloat"),
        "fkey_to_auth_unique" => category!("dblint/splinter/fkeyToAuthUnique"),
        "extension_versions_outdated" => category!("dblint/splinter/extensionVersionsOutdated"),
        _ => category!("dblint/splinter/unknown"),
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
