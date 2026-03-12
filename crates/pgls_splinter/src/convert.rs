use pgls_diagnostics::{DatabaseObjectOwned, Severity};
use serde_json::Value;

use crate::{SplinterAdvices, SplinterDiagnostic, SplinterQueryResult, registry};

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

        let category = registry::get_rule_category(&result.name).unwrap_or_else(|| {
            eprintln!("Warning: Unknown splinter rule: {group}/{}", result.name);
            pgls_diagnostics::category!("splinter")
        });

        SplinterDiagnostic {
            category,
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
