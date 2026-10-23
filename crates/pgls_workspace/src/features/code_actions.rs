use crate::workspace::StatementId;
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;
use pgls_text_size::TextSize;

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
pub struct EditAction {}

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

/// The most rows an executed statement returns; the rest are dropped and the result is marked
/// as truncated.
pub const EXECUTE_STATEMENT_MAX_ROWS: usize = 100;

/// The outcome of executing a statement.
///
/// A statement that returns rows (`select`, `values`, `returning`, `show`, `explain`, ...) fills
/// `columns` and `rows`. Any other statement sets `rows_affected` instead.
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExecuteStatementResult {
    /// A short summary for the user.
    pub message: String,
    /// The names of the returned columns.
    #[serde(default)]
    pub columns: Vec<String>,
    /// The returned rows, at most [`EXECUTE_STATEMENT_MAX_ROWS`]. Each value is in its Postgres
    /// text representation, as `psql` shows it, and `null` for SQL `NULL`.
    #[serde(default)]
    pub rows: Vec<Vec<Option<String>>>,
    /// Whether the statement returned more rows than `rows` holds.
    #[serde(default)]
    pub truncated: bool,
    /// The number of rows affected, for a statement that returns no rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_affected: Option<u64>,
    /// Why Postgres rejected the statement, if it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecuteStatementError>,
}

/// An error that Postgres raised while executing a statement.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExecuteStatementError {
    /// The primary error message, e.g. `division by zero`.
    pub message: String,
    /// The SQLSTATE error code, e.g. `22012`.
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The offset of the error in the document, if Postgres reported a position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<TextSize>,
}
