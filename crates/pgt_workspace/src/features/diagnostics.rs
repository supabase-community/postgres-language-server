use pgt_analyse::RuleCategories;
use pgt_configuration::RuleSelector;
use pgt_fs::PgTPath;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsParams {
    pub path: PgTPath,
    pub categories: RuleCategories,
    pub max_diagnostics: u64,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsResult {
    pub diagnostics: Vec<pgt_diagnostics::serde::Diagnostic>,
    pub errors: usize,
    pub skipped_diagnostics: u64,
}
