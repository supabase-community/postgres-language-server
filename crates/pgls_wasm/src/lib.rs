//! WASM bindings for the Postgres Language Server.
//!
//! This crate provides a WebAssembly interface for using the language server
//! in browser environments.
//!
//! # Architecture
//!
//! The WASM binding exposes two main structs:
//! - `MemoryFs`: An in-memory file system for storing SQL files
//! - `Workspace`: The main API for parsing SQL (backed by pgls_workspace)
//!
//! # Example
//!
//! ```ignore
//! // JavaScript/TypeScript usage (via Emscripten bindings)
//! const workspace = new Workspace();
//! workspace.set_schema(schemaJson);
//! workspace.insert_file("/query.sql", "SELECT * FROM users;");
//! const errors = workspace.parse("SELECT * FROM users;");
//! ```

use std::path::PathBuf;

use pgls_analyse::RuleCategories;
use pgls_configuration::{PartialConfiguration, PartialFormatConfiguration};
use pgls_diagnostics::Diagnostic as _;
use pgls_fs::PgLSPath;
use pgls_text_size::TextSize;
use pgls_workspace::Workspace as WorkspaceTrait;
use pgls_workspace::features::code_actions::{
    CodeAction, CodeActionsParams, FixFileMode, FixFileParams, FixFileResult,
};
use pgls_workspace::features::completions::GetCompletionsParams;
use pgls_workspace::features::diagnostics::PullFileDiagnosticsParams;
use pgls_workspace::features::format::PullFileFormattingParams;
use pgls_workspace::features::on_hover::OnHoverParams;
use pgls_workspace::workspace::{
    ChangeFileParams, CloseFileParams, OpenFileParams, ProjectKey, RegisterProjectFolderParams,
    UpdateSettingsParams,
};
use serde::{Deserialize, Serialize};

mod error;
pub mod ffi;
pub mod lsp;

pub use error::WasmError;
pub use lsp::LspHandler;

/// Simplified diagnostic for WASM output.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Start offset in the file
    pub start: u32,
    /// End offset in the file
    pub end: u32,
    /// The diagnostic message
    pub message: String,
    /// Severity: "error", "warning", "info"
    pub severity: String,
    /// The category, e.g. `lint/banDropTable` or `syntax`
    pub category: String,
}

impl From<pgls_diagnostics::serde::Diagnostic> for Diagnostic {
    fn from(d: pgls_diagnostics::serde::Diagnostic) -> Self {
        let span = d.span().unwrap_or_default();
        Diagnostic {
            start: u32::from(span.start()),
            end: u32::from(span.end()),
            message: d.description_text().to_string(),
            severity: match d.get_severity() {
                pgls_diagnostics::Severity::Error | pgls_diagnostics::Severity::Fatal => {
                    "error".to_string()
                }
                pgls_diagnostics::Severity::Warning => "warning".to_string(),
                pgls_diagnostics::Severity::Information => "info".to_string(),
                pgls_diagnostics::Severity::Hint => "hint".to_string(),
            },
            category: d
                .category()
                .map(|category| category.name().to_string())
                .unwrap_or_default(),
        }
    }
}

/// Result of formatting SQL.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatResult {
    /// The formatted SQL. Statements that could not be formatted are kept verbatim.
    pub formatted: String,
    /// Diagnostics for statements that could not be formatted.
    pub diagnostics: Vec<Diagnostic>,
}

/// Virtual path used by [`Workspace::format_sql`] for the transient document.
const FORMAT_SQL_PATH: &str = "/__pgls_format_sql__.sql";

/// Parse formatter options from JSON.
///
/// The JSON mirrors the `format` section of `postgres-language-server.jsonc`
/// (camelCase keys, same defaults). An empty string means "use defaults".
fn parse_format_options(options_json: &str) -> Result<PartialFormatConfiguration, WasmError> {
    if options_json.trim().is_empty() {
        return Ok(PartialFormatConfiguration::default());
    }
    serde_json::from_str(options_json)
        .map_err(|e| WasmError::InvalidOptions(format!("Invalid format options: {e}")))
}

/// Simple completion item for WASM output.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    /// The label to display
    pub label: String,
    /// The kind of completion (table, column, function, etc.)
    pub kind: String,
    /// Optional detail text
    pub detail: Option<String>,
}

/// A single SQL statement extracted from a multi-statement source.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statement {
    /// The SQL text of the statement
    pub sql: String,
    /// Start byte offset in the original SQL string
    pub start: u32,
    /// End byte offset in the original SQL string
    pub end: u32,
}

/// In-memory file system for storing SQL files.
///
/// This wraps `pgls_fs::MemoryFileSystem` with a simpler API
/// suitable for WASM consumers.
pub struct MemoryFs {
    inner: pgls_fs::MemoryFileSystem,
}

impl Default for MemoryFs {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryFs {
    /// Create a new empty file system.
    pub fn new() -> Self {
        Self {
            inner: pgls_fs::MemoryFileSystem::default(),
        }
    }

    /// Insert or update a file in the file system.
    ///
    /// # Arguments
    /// * `path` - The virtual file path (e.g., "/query.sql")
    /// * `content` - The file content as a string
    pub fn insert(&mut self, path: &str, content: &str) {
        self.inner.insert(PathBuf::from(path), content.as_bytes());
    }

    /// Remove a file from the file system.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to remove
    pub fn remove(&mut self, path: &str) {
        self.inner.remove(&PathBuf::from(path));
    }

    /// Read a file's content.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to read
    ///
    /// # Returns
    /// The file content, or an error if the file doesn't exist.
    pub fn read(&self, path: &str) -> Result<String, WasmError> {
        use pgls_fs::FileSystemExt;

        let path_buf = PathBuf::from(path);
        let mut file = self
            .inner
            .open(&path_buf)
            .map_err(|e| WasmError::FileNotFound(format!("Failed to open file '{path}': {e}")))?;

        let mut content = String::new();
        pgls_fs::File::read_to_string(&mut *file, &mut content)
            .map_err(|e| WasmError::FileNotFound(format!("Failed to read file '{path}': {e}")))?;

        Ok(content)
    }
}

/// Main workspace API for language server features.
///
/// Provides methods for parsing and SQL analysis.
/// Backed by `pgls_workspace::WorkspaceServer` for full linting capabilities.
pub struct Workspace {
    inner: pgls_workspace::workspace::server::WorkspaceServer,
    /// The project that every file belongs to
    project_key: ProjectKey,
    fs: MemoryFs,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// Create a new workspace with an empty file system and no schema.
    pub fn new() -> Self {
        let inner = pgls_workspace::workspace::server::WorkspaceServer::new();

        // Register a project without a folder, which every file belongs to
        let project_key = inner
            .register_project_folder(RegisterProjectFolderParams { path: None })
            .unwrap_or_default();

        Self {
            inner,
            project_key,
            fs: MemoryFs::new(),
        }
    }

    /// Set the database schema from JSON.
    ///
    /// The JSON should match the structure of `Snapshot`.
    /// This is typically exported using `postgres-language-server schema-export`.
    ///
    /// # Arguments
    /// * `json` - JSON string representation of the database snapshot
    ///
    /// # Returns
    /// An error if the JSON is invalid.
    pub fn set_schema(&self, json: &str) -> Result<(), WasmError> {
        self.inner
            .set_schema_json(json)
            .map_err(|e| WasmError::InvalidSchema(format!("Failed to set schema: {e}")))
    }

    /// Clear the current schema.
    pub fn clear_schema(&self) {
        self.inner.clear_schema();
    }

    /// Insert or update a file in the workspace.
    ///
    /// # Arguments
    /// * `path` - The virtual file path (e.g., "/query.sql")
    /// * `content` - The file content as a string
    pub fn insert_file(&mut self, path: &str, content: &str) {
        self.fs.insert(path, content);

        // Also open in the inner workspace
        let pgls_path = PgLSPath::new(PathBuf::from(path));
        let _ = self.inner.open_file(OpenFileParams {
            project_key: self.project_key,
            path: pgls_path.clone(),
            content: content.into(),
            version: 0,
        });

        // Update the content
        let _ = self.inner.change_file(ChangeFileParams {
            project_key: self.project_key,
            path: pgls_path,
            content: content.into(),
            version: 1,
        });
    }

    /// Remove a file from the workspace.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to remove
    pub fn remove_file(&mut self, path: &str) {
        self.fs.remove(path);

        let pgls_path = PgLSPath::new(PathBuf::from(path));
        let _ = self.inner.close_file(CloseFileParams {
            project_key: self.project_key,
            path: pgls_path,
        });
    }

    /// Read a file's content.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to read
    pub fn read_file(&self, path: &str) -> Result<String, WasmError> {
        self.fs.read(path)
    }

    /// Lint a file and return diagnostics.
    ///
    /// Uses the full pgls_workspace linter for comprehensive diagnostics
    /// including lint rules, not just parse errors.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to lint
    ///
    /// # Returns
    /// A list of diagnostics, or an error if the file doesn't exist.
    pub fn lint(&self, path: &str) -> Result<Vec<Diagnostic>, WasmError> {
        // Verify file exists
        let _content = self.fs.read(path)?;

        let pgls_path = PgLSPath::new(PathBuf::from(path));
        let result = self
            .inner
            .pull_file_diagnostics(PullFileDiagnosticsParams {
                project_key: self.project_key,
                path: pgls_path,
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .map_err(|e| WasmError::ParseError(format!("Lint failed: {e}")))?;

        // Convert workspace diagnostics to our simplified format
        let diagnostics = result
            .diagnostics
            .into_iter()
            .map(Diagnostic::from)
            .collect();

        Ok(diagnostics)
    }

    /// The code actions at `offset`: the fixes of the lint diagnostics there and the actions that
    /// suppress them. There is no database connection, so there are no commands.
    pub fn code_actions(&self, path: &str, offset: u32) -> Result<Vec<CodeAction>, WasmError> {
        self.fs.read(path)?;
        let result = self
            .inner
            .pull_code_actions(CodeActionsParams {
                path: PgLSPath::new(PathBuf::from(path)),
                cursor_position: TextSize::from(offset),
                only: vec![],
                skip: vec![],
            })
            .map_err(|e| WasmError::ParseError(format!("Code actions failed: {e}")))?;
        Ok(result.actions)
    }

    /// The content of the file with all safe fixes applied, and with the unsafe ones too if
    /// `unsafe_fixes` is set.
    pub fn fix_file(&self, path: &str, unsafe_fixes: bool) -> Result<FixFileResult, WasmError> {
        self.fs.read(path)?;
        self.inner
            .fix_file(FixFileParams {
                path: PgLSPath::new(PathBuf::from(path)),
                fix_file_mode: if unsafe_fixes {
                    FixFileMode::SafeAndUnsafeFixes
                } else {
                    FixFileMode::SafeFixes
                },
                only: vec![],
                skip: vec![],
            })
            .map_err(|e| WasmError::ParseError(format!("Fixing failed: {e}")))
    }

    /// Format a file in the workspace.
    ///
    /// Formatting is always enabled for explicit calls, regardless of the
    /// `enabled` value in the options.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to format
    /// * `options_json` - Formatter options as JSON, mirroring the `format` section of the
    ///   configuration file (e.g. `{"keywordCase":"upper"}`). Pass an empty string for defaults.
    ///
    /// # Returns
    /// The formatted SQL and diagnostics for statements that could not be formatted,
    /// or an error if the file doesn't exist or the options are invalid.
    pub fn format(&self, path: &str, options_json: &str) -> Result<FormatResult, WasmError> {
        // Verify file exists
        let _content = self.fs.read(path)?;

        self.format_open_document(PgLSPath::new(PathBuf::from(path)), options_json)
    }

    /// Format a SQL string without adding it to the workspace.
    ///
    /// See [`Workspace::format`] for the options format.
    pub fn format_sql(&self, sql: &str, options_json: &str) -> Result<FormatResult, WasmError> {
        let pgls_path = PgLSPath::new(PathBuf::from(FORMAT_SQL_PATH));
        self.inner
            .open_file(OpenFileParams {
                project_key: self.project_key,
                path: pgls_path.clone(),
                content: sql.into(),
                version: 0,
            })
            .map_err(|e| WasmError::Internal(format!("Failed to open document: {e}")))?;

        let result = self.format_open_document(pgls_path.clone(), options_json);

        let _ = self.inner.close_file(CloseFileParams {
            project_key: self.project_key,
            path: pgls_path,
        });

        result
    }

    fn format_open_document(
        &self,
        path: PgLSPath,
        options_json: &str,
    ) -> Result<FormatResult, WasmError> {
        let mut format = parse_format_options(options_json)?;
        format.enabled = Some(true);
        // File matching does not apply to in-memory formatting.
        format.ignore = None;
        format.include = None;

        self.inner
            .update_settings(UpdateSettingsParams {
                project_key: self.project_key,
                configuration: PartialConfiguration {
                    format: Some(format),
                    ..Default::default()
                },
                vcs_base_path: None,
                gitignore_matches: vec![],
                workspace_directory: None,
            })
            .map_err(|e| {
                WasmError::InvalidOptions(format!("Failed to apply format options: {e}"))
            })?;

        let result = self
            .inner
            .pull_file_formatting(PullFileFormattingParams {
                project_key: self.project_key,
                path,
                range: None,
            })
            .map_err(|e| WasmError::Internal(format!("Format failed: {e}")))?;

        Ok(FormatResult {
            formatted: result.formatted,
            diagnostics: result
                .diagnostics
                .into_iter()
                .map(Diagnostic::from)
                .collect(),
        })
    }

    /// Get completions at a position in a file.
    ///
    /// Uses the full pgls_workspace completions engine.
    ///
    /// # Arguments
    /// * `path` - The virtual file path
    /// * `offset` - The byte offset in the file
    ///
    /// # Returns
    /// A list of completion items, or an error if the file doesn't exist.
    pub fn complete(&self, path: &str, offset: u32) -> Result<Vec<CompletionItem>, WasmError> {
        // Verify file exists
        let _content = self.fs.read(path)?;

        let pgls_path = PgLSPath::new(PathBuf::from(path));
        let result = self
            .inner
            .get_completions(GetCompletionsParams {
                project_key: self.project_key,
                path: pgls_path,
                position: TextSize::from(offset),
            })
            .map_err(|e| WasmError::ParseError(format!("Completions failed: {e}")))?;

        // Convert workspace completions to our simplified format
        let items = result
            .into_iter()
            .map(|c| {
                let kind = match c.kind {
                    pgls_completions::CompletionItemKind::Table => "table",
                    pgls_completions::CompletionItemKind::Column => "column",
                    pgls_completions::CompletionItemKind::Function => "function",
                    pgls_completions::CompletionItemKind::Schema => "schema",
                    pgls_completions::CompletionItemKind::Policy => "policy",
                    pgls_completions::CompletionItemKind::Role => "role",
                    pgls_completions::CompletionItemKind::Keyword => "keyword",
                };
                CompletionItem {
                    label: c.label,
                    kind: kind.to_string(),
                    detail: Some(c.description),
                }
            })
            .collect();

        Ok(items)
    }

    /// Get hover information at a position in a file.
    ///
    /// Uses the full pgls_workspace hover engine.
    ///
    /// # Arguments
    /// * `path` - The virtual file path
    /// * `offset` - The byte offset in the file
    ///
    /// # Returns
    /// Hover text (markdown formatted), or None if no hover info is available.
    pub fn hover(&self, path: &str, offset: u32) -> Result<Option<String>, WasmError> {
        // Verify file exists
        let _content = self.fs.read(path)?;

        let pgls_path = PgLSPath::new(PathBuf::from(path));
        let result = self
            .inner
            .on_hover(OnHoverParams {
                project_key: self.project_key,
                path: pgls_path,
                position: TextSize::from(offset),
            })
            .map_err(|e| WasmError::ParseError(format!("Hover failed: {e}")))?;

        let blocks: Vec<String> = result.into_iter().collect();
        if blocks.is_empty() {
            Ok(None)
        } else {
            Ok(Some(blocks.join("\n\n")))
        }
    }

    /// Parse SQL and return parse errors (if any).
    ///
    /// # Arguments
    /// * `sql` - The SQL string to parse
    ///
    /// # Returns
    /// A list of parse error messages, empty if parsing succeeded.
    pub fn parse(&self, sql: &str) -> Vec<String> {
        match pgls_query::parse(sql) {
            Ok(_) => vec![],
            Err(e) => vec![e.to_string()],
        }
    }

    /// Split SQL into individual statements.
    ///
    /// Each returned `Statement` contains the SQL text and its byte offsets
    /// in the original string.
    pub fn split_statements(&self, sql: &str) -> Vec<Statement> {
        self.inner
            .split_statements(sql)
            .into_iter()
            .map(|range| Statement {
                sql: sql[range].to_string(),
                start: u32::from(range.start()),
                end: u32::from(range.end()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_basic() {
        let mut workspace = Workspace::new();
        workspace.insert_file("/test.sql", "SELECT * FROM users;");

        // Should be able to lint without schema
        let diagnostics = workspace.lint("/test.sql").unwrap();
        // No diagnostics expected for valid SQL
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_workspace_parse() {
        let workspace = Workspace::new();

        // Valid SQL
        let errors = workspace.parse("SELECT 1;");
        assert!(errors.is_empty());

        // Invalid SQL
        let errors = workspace.parse("SELEC 1;");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_workspace_split_statements() {
        let workspace = Workspace::new();

        let statements = workspace.split_statements("SELECT 1; SELECT 2;");
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1;");
        assert_eq!(statements[0].start, 0);
        assert_eq!(statements[0].end, 9);
        assert_eq!(statements[1].sql, "SELECT 2;");
        assert_eq!(statements[1].start, 10);
        assert_eq!(statements[1].end, 19);
    }

    #[test]
    fn test_workspace_split_statements_ignores_comments() {
        let workspace = Workspace::new();

        let statements = workspace.split_statements("-- comment\nSELECT 1;\n\nSELECT 2;");
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1;");
        assert_eq!(statements[1].sql, "SELECT 2;");
    }

    const FORMAT_INPUT: &str = r#"CREATE TABLE "public"."t" ("id" "uuid" NOT NULL)"#;

    #[test]
    fn test_workspace_format_sql_keyword_case_upper() {
        let workspace = Workspace::new();

        let result = workspace
            .format_sql(FORMAT_INPUT, r#"{"keywordCase":"upper"}"#)
            .unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(
            result.formatted,
            "CREATE TABLE public.t (id uuid NOT NULL);"
        );
    }

    #[test]
    fn test_workspace_format_sql_defaults() {
        let workspace = Workspace::new();

        let result = workspace.format_sql(FORMAT_INPUT, "").unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(
            result.formatted,
            "create table public.t (id uuid not null);"
        );

        // Options from a previous call must not leak into the next one.
        workspace
            .format_sql(FORMAT_INPUT, r#"{"keywordCase":"upper"}"#)
            .unwrap();
        let result = workspace.format_sql(FORMAT_INPUT, "{}").unwrap();
        assert_eq!(
            result.formatted,
            "create table public.t (id uuid not null);"
        );
    }

    #[test]
    fn test_workspace_format_ignores_enabled_false() {
        let workspace = Workspace::new();

        let result = workspace
            .format_sql(FORMAT_INPUT, r#"{"enabled":false}"#)
            .unwrap();
        assert_eq!(
            result.formatted,
            "create table public.t (id uuid not null);"
        );
    }

    #[test]
    fn test_workspace_format_file() {
        let mut workspace = Workspace::new();
        workspace.insert_file("/format.sql", "select 1;\n\nSELEC 2;\n");

        let result = workspace
            .format("/format.sql", r#"{"keywordCase":"upper"}"#)
            .unwrap();
        assert_eq!(result.formatted, "SELECT 1;\n\nSELEC 2;\n");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, "error");

        assert!(workspace.format("/missing.sql", "").is_err());
    }

    #[test]
    fn test_workspace_format_invalid_options() {
        let workspace = Workspace::new();

        assert!(matches!(
            workspace.format_sql("select 1", r#"{"keywordCase":"shout"}"#),
            Err(WasmError::InvalidOptions(_))
        ));
        assert!(matches!(
            workspace.format_sql("select 1", r#"{"unknownOption":true}"#),
            Err(WasmError::InvalidOptions(_))
        ));
    }

    #[test]
    fn test_memory_fs() {
        let mut fs = MemoryFs::new();
        fs.insert("/test.sql", "SELECT 1;");

        let content = fs.read("/test.sql").unwrap();
        assert_eq!(content, "SELECT 1;");

        fs.remove("/test.sql");
        assert!(fs.read("/test.sql").is_err());
    }
}
