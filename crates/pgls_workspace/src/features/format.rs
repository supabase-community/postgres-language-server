use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullFileFormattingParams {
    pub path: PgLSPath,
    /// Optional range to format. If None, format the entire file.
    pub range: Option<TextRange>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatementFormatResult {
    pub original: String,
    pub formatted: String,
    pub range: TextRange,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullFormattingResult {
    pub original: String,
    pub formatted: String,
    pub statements: Vec<StatementFormatResult>,
    pub diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
}
