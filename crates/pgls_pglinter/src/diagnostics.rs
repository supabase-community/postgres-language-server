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
}
