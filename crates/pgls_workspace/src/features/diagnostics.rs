use pgls_analyse::RuleCategories;
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullFileDiagnosticsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    pub categories: RuleCategories,
    pub max_diagnostics: u32,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsResult {
    pub diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
    pub skipped_diagnostics: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CheckDatabaseConnectionParams {
    /// The project whose databases are checked: the one of its root settings and of every
    /// nested configuration file.
    pub project_key: crate::workspace::ProjectKey,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CheckDatabaseConnectionResult {
    /// Why the configured database cannot be used, if it cannot.
    pub error: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDatabaseDiagnosticsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub categories: RuleCategories,
    pub max_diagnostics: u32,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}
