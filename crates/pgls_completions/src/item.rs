use std::fmt::Display;

use pgls_text_size::TextRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum CompletionItemKind {
    Table,
    Function,
    Column,
    Schema,
    Policy,
    Role,
}

impl Display for CompletionItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            CompletionItemKind::Table => "Table",
            CompletionItemKind::Function => "Function",
            CompletionItemKind::Column => "Column",
            CompletionItemKind::Schema => "Schema",
            CompletionItemKind::Policy => "Policy",
            CompletionItemKind::Role => "Role",
        };

        write!(f, "{txt}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
/// The text that the editor should fill in.
/// If `None`, the `label` should be used.
/// Tables, for example, might have different completion_texts:
///
/// label: "users", description: "Schema: auth", completion_text: "auth.users".
pub struct CompletionText {
    pub text: String,
    /// A `range` is required because some editors replace the current token,
    /// others naively insert the text.
    /// Having a range where start == end makes it an insertion.
    pub range: TextRange,

    pub is_snippet: bool,
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
    pub detail: Option<String>,

    pub completion_text: Option<CompletionText>,
}
