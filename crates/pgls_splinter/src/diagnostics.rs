use pgls_diagnostics::{
    Advices, Category, Diagnostic, LogCategory, MessageAndDescription, Severity, Visit,
};
use serde_json::Value;
use std::io;

/// A specialized diagnostic for Splinter (database-level linting).
#[derive(Debug, Diagnostic, PartialEq)]
pub struct SplinterDiagnostic {
    #[category]
    pub category: &'static Category,

    // TODO: add new location type for database objects
    // This will map schema + object_name to source code location
    // #[location(span)]
    // pub span: Option<TextRange>,
    #[message]
    #[description]
    pub message: MessageAndDescription,

    #[severity]
    pub severity: Severity,

    #[advice]
    pub advices: SplinterAdvices,
}

/// Advices for Splinter diagnostics, containing database-level issue details
#[derive(Debug, PartialEq)]
pub struct SplinterAdvices {
    /// General description of what this rule detects
    pub description: String,

    /// Database schema name (e.g., "public", "auth")
    pub schema: Option<String>,

    /// Database object name (e.g., table name, view name, function name)
    pub object_name: Option<String>,

    /// Type of database object (e.g., "table", "view", "materialized view", "function")
    pub object_type: Option<String>,

    /// URL to documentation/remediation guide
    pub remediation_url: String,

    /// Additional rule-specific metadata (e.g., fkey_name, column, indexes)
    /// This contains fields that don't fit into the common structure
    pub additional_metadata: Option<Value>,
}

impl Advices for SplinterAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        // Show rule description
        visitor.record_log(LogCategory::None, &self.description)?;

        // Show affected database object
        if let (Some(schema), Some(name)) = (&self.schema, &self.object_name) {
            let type_str = self
                .object_type
                .as_ref()
                .map(|t| format!("{t}: "))
                .unwrap_or_default();
            visitor.record_log(LogCategory::Info, &format!("{type_str}{schema}.{name}"))?;
        }

        // Show additional metadata if present
        if let Some(metadata) = &self.additional_metadata {
            if !metadata.is_null() {
                visitor.record_log(LogCategory::None, &format!("{metadata}"))?;
            }
        }

        // Show remediation URL
        visitor.record_log(
            LogCategory::Info,
            &format!("Documentation: {}", &self.remediation_url),
        )?;

        Ok(())
    }
}
