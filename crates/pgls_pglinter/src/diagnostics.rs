//! Pglinter diagnostic types and conversion from SARIF

use pgls_diagnostics::{
    Advices, Category, DatabaseObjectOwned, Diagnostic, LogCategory, MessageAndDescription,
    Severity, Visit,
};
use std::io;

use crate::sarif;

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

/// Error when converting SARIF to diagnostics
#[derive(Debug)]
pub struct UnknownRuleError {
    pub rule_code: String,
}

impl std::fmt::Display for UnknownRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown pglinter rule code: {}", self.rule_code)
    }
}

impl std::error::Error for UnknownRuleError {}

impl PglinterDiagnostic {
    /// Try to convert a single SARIF result to a pglinter diagnostic
    pub fn try_from_sarif(
        result: &sarif::Result,
        rule_code: &str,
    ) -> Result<PglinterDiagnostic, UnknownRuleError> {
        let category =
            crate::registry::get_rule_category(rule_code).ok_or_else(|| UnknownRuleError {
                rule_code: rule_code.to_string(),
            })?;

        let metadata = crate::registry::get_rule_metadata_by_code(rule_code);

        let severity = match result.level_str() {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            "note" => Severity::Information,
            _ => Severity::Warning,
        };

        let message = result.message_text().to_string();
        let description = metadata
            .map(|m| m.description.to_string())
            .unwrap_or_else(|| message.clone());

        let fixes = metadata
            .map(|m| m.fixes.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let object_list = {
            let names = result.logical_location_names();
            if names.is_empty() {
                None
            } else {
                Some(names.join("\n"))
            }
        };

        Ok(PglinterDiagnostic {
            category,
            db_object: None,
            message: message.into(),
            severity,
            advices: PglinterAdvices {
                description,
                rule_code: Some(rule_code.to_string()),
                fixes,
                object_list,
            },
        })
    }

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
}
