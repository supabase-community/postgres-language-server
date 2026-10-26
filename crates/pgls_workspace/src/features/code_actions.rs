//! The code actions offered at the cursor, and the commands and fixes they run: lint fixes and
//! suppressions (`lint`), expanding a `*` into its columns (`expand_star`), executing the
//! statement (`execute_statement`), and invalidating the schema cache (`schema_cache`).

use crate::workspace::StatementId;
use pgls_analyse::{ActionCategory, Applicability};
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};

pub(crate) mod execute_statement;
pub(crate) mod expand_star;
pub(crate) mod lint;
pub(crate) mod schema_cache;

pub use execute_statement::{
    EXECUTE_STATEMENT_MAX_ROWS, ExecuteStatementError, ExecuteStatementParams,
    ExecuteStatementResult,
};
pub use lint::{FixFileMode, FixFileParams, FixFileResult};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeActionsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    pub cursor_position: TextSize,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeActionsResult {
    pub actions: Vec<CodeAction>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CodeActionKind {
    Edit(EditAction),
    Command(CommandAction),
    EditAndCommand(EditAction, CommandAction),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EditAction {
    /// The category of the diagnostic the action fixes or suppresses, e.g. `lint/banDropTable`.
    /// `None` for refactors.
    pub rule: Option<String>,
    pub category: ActionCategory,
    pub applicability: Applicability,
    /// Non-overlapping edits of the file, ordered by their position.
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TextEdit {
    pub range: TextRange,
    pub new_text: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CommandAction {
    pub category: CommandActionCategory,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CommandActionCategory {
    ExecuteStatement(StatementId),
    InvalidateSchemaCache,
}
