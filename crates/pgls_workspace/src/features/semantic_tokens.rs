use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;

// Re-export from the workspace implementation
pub use crate::workspace::server::semantic_tokens::{
    SemanticToken, TokenModifier, TokenType, NO_MODIFIERS,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SemanticTokensParams {
    /// The file path for which semantic tokens are requested.
    pub path: PgLSPath,
    /// Optional range to limit the tokens. If None, returns tokens for the entire file.
    pub range: Option<TextRange>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SemanticTokensResult {
    /// The semantic tokens for the requested file/range
    pub tokens: Vec<SemanticToken>,
}

impl SemanticTokensResult {
    pub fn new(tokens: Vec<SemanticToken>) -> Self {
        Self { tokens }
    }
}
