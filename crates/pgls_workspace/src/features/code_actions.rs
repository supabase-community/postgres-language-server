use crate::workspace::StatementId;
use pgls_analyse::{ActionCategory, Applicability};
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeActionsParams {
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
    pub rule: String,
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExecuteStatementParams {
    pub statement_id: StatementId,
    pub path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExecuteStatementResult {
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FixFileParams {
    pub path: PgLSPath,
    pub fix_file_mode: FixFileMode,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

/// Which fixes `fix_file` applies.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FixFileMode {
    /// Only the fixes that are safe to apply.
    SafeFixes,
    /// The safe and the unsafe fixes.
    SafeAndUnsafeFixes,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FixFileResult {
    /// The content of the file with the fixes applied.
    pub code: String,
    /// The number of fixes that were applied.
    pub fixes_applied: u32,
    /// The number of unsafe fixes that were not applied with [FixFileMode::SafeFixes].
    pub skipped_suggested_fixes: u32,
}
