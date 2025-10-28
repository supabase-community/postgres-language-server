use pgls_analyse::RuleCategories;
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsParams {
    pub path: PgLSPath,
    pub categories: RuleCategories,
    pub max_diagnostics: u64,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsResult {
    pub diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
    pub errors: usize,
    pub skipped_diagnostics: u64,
}
