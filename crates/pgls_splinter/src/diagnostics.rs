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
    pub remediation: String,

    /// Additional rule-specific metadata (e.g., fkey_name, column, indexes)
    /// This contains fields that don't fit into the common structure
    pub additional_metadata: Option<Value>,
}

impl Advices for SplinterAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        // Show rule description
        visitor.record_log(LogCategory::None, &self.description)?;

        // Show affected database object, also when it has no schema (e.g. an extension)
        if let Some(name) = &self.object_name {
            let type_str = self
                .object_type
                .as_ref()
                .map(|t| format!("{t}: "))
                .unwrap_or_default();
            let schema_str = self
                .schema
                .as_ref()
                .map(|schema| format!("{schema}."))
                .unwrap_or_default();
            visitor.record_log(LogCategory::Info, &format!("{type_str}{schema_str}{name}"))?;
        }

        // Show additional metadata if present
        if let Some(metadata) = &self.additional_metadata
            && !metadata.is_null()
        {
            visitor.record_log(LogCategory::None, &format!("{metadata}"))?;
        }

        // Show remediation
        visitor.record_log(
            LogCategory::Info,
            &format!("Remediation: {}", self.remediation),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pgls_diagnostics::{Error, Severity, print_diagnostic_to_string};

    use super::{SplinterAdvices, SplinterDiagnostic};

    fn printed(schema: Option<&str>) -> String {
        let diagnostic = SplinterDiagnostic {
            category: pgls_diagnostics::category!("splinter"),
            message: "Extension `pg_net` is installed in the public schema".into(),
            severity: Severity::Warning,
            advices: SplinterAdvices {
                description: "Detects extensions installed in the public schema.".into(),
                schema: schema.map(String::from),
                object_name: Some("pg_net".into()),
                object_type: Some("extension".into()),
                remediation: "https://supabase.com/docs".into(),
                additional_metadata: None,
            },
        };
        print_diagnostic_to_string(&Error::from(diagnostic))
    }

    #[test]
    fn object_with_schema() {
        let printed = printed(Some("public"));
        assert!(printed.contains("extension: public.pg_net"), "{printed}");
    }

    #[test]
    fn object_without_schema() {
        let printed = printed(None);
        assert!(printed.contains("extension: pg_net"), "{printed}");
    }
}
