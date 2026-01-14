//! Pglinter diagnostic types

use pgls_diagnostics::{
    Advices, Category, DatabaseObjectOwned, Diagnostic, LogCategory, MessageAndDescription,
    Severity, Visit,
};
use std::io;

/// A specialized diagnostic for pglinter (database-level linting via pglinter extension).
#[derive(Debug, Diagnostic, PartialEq)]
pub struct PglinterDiagnostic {
    #[category]
    pub category: &'static Category,

    #[location(database_object)]
    pub db_object: Option<DatabaseObjectOwned>,

    #[message]
    #[description]
    pub message: MessageAndDescription,

    #[severity]
    pub severity: Severity,

    #[advice]
    pub advices: PglinterAdvices,
}

/// Advices for pglinter diagnostics
#[derive(Debug, PartialEq)]
pub struct PglinterAdvices {
    /// General description of what this rule detects
    pub description: String,

    /// Rule code (e.g., "B001", "S001", "C001")
    pub rule_code: Option<String>,

    /// Suggested fixes for the issue
    pub fixes: Vec<String>,

    /// List of affected database objects
    pub object_list: Option<String>,
}

impl Advices for PglinterAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        if !self.description.is_empty() {
            visitor.record_log(LogCategory::None, &self.description)?;
        }

        if let Some(code) = &self.rule_code {
            visitor.record_log(LogCategory::Info, &format!("Rule: {code}"))?;
        }

        if let Some(objects) = &self.object_list {
            if !objects.is_empty() {
                visitor.record_log(LogCategory::None, &"Affected objects:")?;
                for line in objects.lines() {
                    visitor.record_log(LogCategory::Info, &format!("  {line}"))?;
                }
            }
        }

        if !self.fixes.is_empty() {
            visitor.record_log(LogCategory::None, &"How to fix:")?;
            for (i, fix) in self.fixes.iter().enumerate() {
                let num = i + 1;
                visitor.record_log(LogCategory::Info, &format!("  {num}. {fix}"))?;
            }
        }

        Ok(())
    }
}

impl PglinterDiagnostic {
    /// Create diagnostic for missing pglinter extension
    pub fn extension_not_installed() -> PglinterDiagnostic {
        PglinterDiagnostic {
            category: pgls_diagnostics::category!("pglinter/extensionNotInstalled"),
            db_object: None,
            message: "The pglinter extension is not installed in the database. Install it with 'CREATE EXTENSION pglinter' or disable pglinter rules in your configuration.".into(),
            severity: Severity::Error,
            advices: PglinterAdvices {
                description: "pglinter rules are enabled in your configuration but the extension is not installed.".to_string(),
                rule_code: None,
                fixes: vec!["Install the pglinter extension: CREATE EXTENSION pglinter".to_string()],
                object_list: None,
            },
        }
    }

    /// Create diagnostic for rule disabled in pglinter extension
    pub fn rule_disabled_in_extension(rule_code: &str) -> PglinterDiagnostic {
        let description = format!(
            "Rule {rule_code} is enabled in configuration but disabled in pglinter extension. Enable it with: SELECT pglinter.enable_rule('{rule_code}')"
        );

        PglinterDiagnostic {
            category: pgls_diagnostics::category!("pglinter/ruleDisabledInExtension"),
            db_object: None,
            message: description.into(),
            severity: Severity::Error,
            advices: PglinterAdvices {
                description: format!(
                    "Rule {rule_code} is configured to run but is disabled in the pglinter extension."
                ),
                rule_code: Some(rule_code.to_string()),
                fixes: vec![format!(
                    "Enable the rule: SELECT pglinter.enable_rule('{rule_code}')"
                )],
                object_list: None,
            },
        }
    }

    /// Create diagnostic from rule code using known metadata
    pub fn from_rule_code(rule_code: &str) -> Option<PglinterDiagnostic> {
        let category = crate::registry::get_rule_category(rule_code)?;
        let metadata = crate::registry::get_rule_metadata_by_code(rule_code)?;

        let fixes: Vec<String> = metadata.fixes.iter().map(|s| s.to_string()).collect();

        Some(PglinterDiagnostic {
            category,
            db_object: None,
            message: metadata.description.into(),
            severity: Severity::Warning,
            advices: PglinterAdvices {
                description: metadata.description.to_string(),
                rule_code: Some(rule_code.to_string()),
                fixes,
                object_list: None,
            },
        })
    }

    /// Create diagnostic from a pglinter violation with optional object info
    pub fn from_violation(
        rule_code: &str,
        db_object: Option<DatabaseObjectOwned>,
    ) -> Option<PglinterDiagnostic> {
        let category = crate::registry::get_rule_category(rule_code)?;
        let metadata = crate::registry::get_rule_metadata_by_code(rule_code)?;

        let fixes: Vec<String> = metadata.fixes.iter().map(|s| s.to_string()).collect();

        // Generate a violation-specific message
        let message = violation_message(rule_code, db_object.as_ref());

        // Generate a violation-specific advice (more detailed explanation)
        let advice_description = violation_advice(rule_code);

        Some(PglinterDiagnostic {
            category,
            db_object,
            message: message.into(),
            severity: Severity::Warning,
            advices: PglinterAdvices {
                description: advice_description,
                rule_code: Some(rule_code.to_string()),
                fixes,
                object_list: None,
            },
        })
    }
}

/// Generate a user-friendly violation message for a specific object
fn violation_message(rule_code: &str, db_object: Option<&DatabaseObjectOwned>) -> String {
    let obj_name = db_object
        .map(|obj| {
            if let Some(ref schema) = obj.schema {
                format!("'{}.{}'", schema, obj.name)
            } else {
                format!("'{}'", obj.name)
            }
        })
        .unwrap_or_else(|| "Object".to_string());

    let obj_type = db_object
        .and_then(|obj| obj.object_type.as_deref())
        .unwrap_or("object");

    match rule_code {
        // Base rules
        "B001" => format!("Table {obj_name} has no primary key"),
        "B002" => format!("Index on {obj_name} is redundant"),
        "B003" => format!("Foreign key on {obj_name} has no index"),
        "B004" => format!("Index on {obj_name} is unused"),
        "B005" => format!(
            "{} {} uses uppercase characters",
            capitalize(obj_type),
            obj_name
        ),
        "B006" => format!("Table {obj_name} is never selected from"),
        "B007" => format!("Foreign key on {obj_name} references table outside its schema"),
        "B008" => format!("Foreign key on {obj_name} has type mismatch"),
        "B009" => format!("Table {obj_name} has duplicate trigger"),
        "B010" => format!(
            "{} {} uses reserved SQL keyword",
            capitalize(obj_type),
            obj_name
        ),
        "B011" => format!("Tables in {obj_name} have different owners"),
        "B012" => format!("Table {obj_name} has composite primary key with too many columns"),
        // Schema rules
        "S001" => format!("Schema {obj_name} has no default role granted"),
        "S002" => format!("Schema {obj_name} name is prefixed/suffixed with environment"),
        "S003" => format!("Schema {obj_name} has insecure public access"),
        "S004" => format!("Schema {obj_name} owner is an internal role"),
        "S005" => format!("Schema {obj_name} owner doesn't match table owners"),
        // Cluster rules
        "C001" | "C002" | "C003" => "Cluster configuration issue".to_string(),
        // Fallback
        _ => format!("{} {} has a violation", capitalize(obj_type), obj_name),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Generate detailed advice for a specific rule violation
fn violation_advice(rule_code: &str) -> String {
    match rule_code {
        "B001" => "Tables without primary keys cannot be uniquely identified, which causes issues with replication, foreign keys, and efficient updates/deletes.".to_string(),
        "B002" => "Redundant indexes waste storage space and slow down write operations without providing query benefits.".to_string(),
        "B003" => "Foreign keys without indexes cause slow cascading operations and inefficient join queries.".to_string(),
        "B004" => "Unused indexes consume storage and slow down writes without benefiting any queries.".to_string(),
        "B005" => "Using uppercase in identifiers requires quoting and can cause case-sensitivity issues.".to_string(),
        "B006" => "Tables never queried may be obsolete and candidates for removal.".to_string(),
        "B007" => "Cross-schema foreign keys can cause issues with schema-level operations and access control.".to_string(),
        "B008" => "Type mismatches in foreign keys can cause implicit casts, affecting performance and data integrity.".to_string(),
        "B009" => "Duplicate triggers may cause unexpected behavior or redundant processing.".to_string(),
        "B010" => "Using SQL reserved keywords as identifiers requires quoting and may cause compatibility issues.".to_string(),
        "B011" => "Mixed ownership in schemas can cause permission and maintenance issues.".to_string(),
        "B012" => "Large composite primary keys are inefficient for indexing and foreign key references.".to_string(),
        "S001" => "Schemas without default role grants may have inconsistent permission patterns.".to_string(),
        "S002" => "Environment prefixes/suffixes in schema names indicate environment-specific configuration that should be handled differently.".to_string(),
        "S003" => "Insecure public access to schemas can expose data to unauthorized users.".to_string(),
        "S004" => "Internal role ownership of schemas can cause maintenance and security issues.".to_string(),
        "S005" => "Mismatched schema/table ownership can cause permission inconsistencies.".to_string(),
        "C001" | "C002" | "C003" => "Cluster configuration issues may affect database stability and performance.".to_string(),
        _ => String::new(),
    }
}
