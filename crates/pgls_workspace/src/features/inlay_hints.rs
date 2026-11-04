use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullInlayHintsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    /// The hints of the statements that overlap this range are returned.
    pub range: TextRange,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullInlayHintsResult {
    pub hints: Vec<InlayHint>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct InlayHint {
    /// The offset in the file that the hint is shown before.
    pub offset: TextSize,
    pub label: String,
    pub kind: InlayHintKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum InlayHintKind {
    /// The name of the parameter that a function argument is passed to.
    Parameter,
}

impl From<pgls_inlay_hints::InlayHintKind> for InlayHintKind {
    fn from(kind: pgls_inlay_hints::InlayHintKind) -> Self {
        match kind {
            pgls_inlay_hints::InlayHintKind::Parameter => Self::Parameter,
        }
    }
}
