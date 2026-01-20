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
use pgls_fs::PgLSPath;
use pgls_text_size::TextSize;
use pgls_workspace::Workspace as WorkspaceTrait;
use pgls_workspace::features::completions::GetCompletionsParams;
use pgls_workspace::features::diagnostics::PullFileDiagnosticsParams;
use pgls_workspace::features::on_hover::OnHoverParams;
use pgls_workspace::workspace::{
    ChangeFileParams, CloseFileParams, OpenFileParams, RegisterProjectFolderParams,
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

        // Register a project folder to enable workspace features
        let _ = inner.register_project_folder(RegisterProjectFolderParams {
            path: None,
            set_as_current_workspace: true,
        });

        Self {
            inner,
            fs: MemoryFs::new(),
        }
    }

    /// Set the database schema from JSON.
    ///
    /// The JSON should match the structure of `SchemaCache`.
    /// This is typically exported using `postgres-language-server schema-export`.
    ///
    /// # Arguments
    /// * `json` - JSON string representation of the schema cache
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
            path: pgls_path.clone(),
            content: content.into(),
            version: 0,
        });

        // Update the content
        let _ = self.inner.change_file(ChangeFileParams {
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
        let _ = self.inner.close_file(CloseFileParams { path: pgls_path });
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
            .map(|d| {
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
                }
            })
            .collect();

        Ok(diagnostics)
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
    fn test_memory_fs() {
        let mut fs = MemoryFs::new();
        fs.insert("/test.sql", "SELECT 1;");

        let content = fs.read("/test.sql").unwrap();
        assert_eq!(content, "SELECT 1;");

        fs.remove("/test.sql");
        assert!(fs.read("/test.sql").is_err());
    }
}
