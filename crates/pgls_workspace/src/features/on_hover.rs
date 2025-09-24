use pgls_fs::PgLSPath;
use pgls_text_size::TextSize;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OnHoverParams {
    pub path: PgLSPath,
    pub position: TextSize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OnHoverResult {
    /// Can contain multiple blocks of markdown
    /// if the hovered-on item is ambiguous.
    pub(crate) markdown_blocks: Vec<String>,
}

impl IntoIterator for OnHoverResult {
    type Item = String;
    type IntoIter = <Vec<String> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.markdown_blocks.into_iter()
    }
}
