//! The command that executes the statement at the cursor against the database.

use pgls_fs::PgLSPath;
use pgls_text_size::TextSize;

use super::{CodeAction, CodeActionKind, CommandAction, CommandActionCategory};
use crate::workspace::{DefaultMapper, Document, StatementId};

#[cfg(feature = "db")]
pub(crate) mod execution;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExecuteStatementParams {
    pub project_key: crate::workspace::ProjectKey,
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

/// The actions that execute the statements at `cursor`. `disabled_reason` says why statement
/// execution isn't allowed, if it isn't.
pub(crate) fn execute_statement_actions(
    doc: &Document,
    cursor: TextSize,
    disabled_reason: Option<String>,
) -> Vec<CodeAction> {
    doc.iter_at(DefaultMapper, cursor)
        .map(|(stmt, _, txt)| {
            let title = format!(
                "Execute Statement: {}...",
                txt.chars().take(50).collect::<String>()
            );

            CodeAction {
                title,
                kind: CodeActionKind::Command(CommandAction {
                    category: CommandActionCategory::ExecuteStatement(stmt),
                }),
                disabled_reason: disabled_reason.clone(),
            }
        })
        .collect()
}
