use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum CompletionItemKind {
    Table,
    Function,
    Column,
    Schema,
}

impl Display for CompletionItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            CompletionItemKind::Table => "Table",
            CompletionItemKind::Function => "Function",
            CompletionItemKind::Column => "Column",
            CompletionItemKind::Schema => "Schema",
        };

        write!(f, "{txt}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompletionItem {
    pub label: String,
    pub description: String,
    pub preselected: bool,
    pub kind: CompletionItemKind,
    /// String used for sorting by LSP clients.
    pub sort_text: String,
}
