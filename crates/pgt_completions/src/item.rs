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
        match self {
            CompletionItemKind::Table => write!(f, "Table"),
            CompletionItemKind::Function => write!(f, "Function"),
            CompletionItemKind::Column => write!(f, "Column"),
            CompletionItemKind::Schema => write!(f, "Schema"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompletionItem {
    pub label: String,
    pub description: String,
    pub preselected: bool,
    pub kind: CompletionItemKind,
    pub score: i32,
}
