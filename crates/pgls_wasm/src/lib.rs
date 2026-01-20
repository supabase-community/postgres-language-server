//! WASM bindings for the Postgres Language Server.
//!
//! This crate provides a WebAssembly interface for using the language server
//! in browser environments.
//!
//! # Architecture
//!
//! The WASM binding exposes two main structs:
//! - `MemoryFs`: An in-memory file system for storing SQL files
//! - `Workspace`: The main API for parsing SQL
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
use std::sync::Arc;

use pgls_schema_cache::SchemaCache;
use serde::{Deserialize, Serialize};

mod error;
pub mod ffi;

pub use error::WasmError;

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
        let mut file = self.inner.open(&path_buf).map_err(|e| {
            WasmError::FileNotFound(format!("Failed to open file '{path}': {e}"))
        })?;

        let mut content = String::new();
        pgls_fs::File::read_to_string(&mut *file, &mut content).map_err(|e| {
            WasmError::FileNotFound(format!("Failed to read file '{path}': {e}"))
        })?;

        Ok(content)
    }
}

/// Main workspace API for language server features.
///
/// Provides methods for parsing and basic SQL analysis.
pub struct Workspace {
    fs: MemoryFs,
    schema: Option<Arc<SchemaCache>>,
    parser: tree_sitter::Parser,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// Create a new workspace with an empty file system and no schema.
    pub fn new() -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Failed to set tree-sitter language");

        Self {
            fs: MemoryFs::new(),
            schema: None,
            parser,
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
    pub fn set_schema(&mut self, json: &str) -> Result<(), WasmError> {
        let schema: SchemaCache = serde_json::from_str(json)
            .map_err(|e| WasmError::InvalidSchema(format!("Failed to parse schema JSON: {e}")))?;
        self.schema = Some(Arc::new(schema));
        Ok(())
    }

    /// Clear the current schema.
    pub fn clear_schema(&mut self) {
        self.schema = None;
    }

    /// Get the current schema (if set).
    pub fn schema(&self) -> Option<&SchemaCache> {
        self.schema.as_deref()
    }

    /// Insert or update a file in the workspace.
    ///
    /// # Arguments
    /// * `path` - The virtual file path (e.g., "/query.sql")
    /// * `content` - The file content as a string
    pub fn insert_file(&mut self, path: &str, content: &str) {
        self.fs.insert(path, content);
    }

    /// Remove a file from the workspace.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to remove
    pub fn remove_file(&mut self, path: &str) {
        self.fs.remove(path);
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
    /// Currently returns parse errors only.
    /// More advanced linting requires additional dependencies.
    ///
    /// # Arguments
    /// * `path` - The virtual file path to lint
    ///
    /// # Returns
    /// A list of diagnostics, or an error if the file doesn't exist.
    pub fn lint(&self, path: &str) -> Result<Vec<Diagnostic>, WasmError> {
        let content = self.fs.read(path)?;

        // Parse with libpg_query
        match pgls_query::parse(&content) {
            Ok(_) => Ok(vec![]),
            Err(e) => {
                // Return parse error as diagnostic
                Ok(vec![Diagnostic {
                    start: 0,
                    end: content.len() as u32,
                    message: e.to_string(),
                    severity: "error".to_string(),
                }])
            }
        }
    }

    /// Get completions at a position in a file.
    ///
    /// Currently provides basic completions from the schema.
    ///
    /// # Arguments
    /// * `path` - The virtual file path
    /// * `offset` - The byte offset in the file
    ///
    /// # Returns
    /// A list of completion items, or an error if the file doesn't exist.
    pub fn complete(&mut self, path: &str, _offset: u32) -> Result<Vec<CompletionItem>, WasmError> {
        // Verify file exists
        let _content = self.fs.read(path)?;

        // Return basic completions from schema
        let mut items = Vec::new();

        if let Some(schema) = &self.schema {
            // Add tables
            for table in &schema.tables {
                items.push(CompletionItem {
                    label: table.name.clone(),
                    kind: "table".to_string(),
                    detail: Some(format!("Table in {}", table.schema)),
                });
            }

            // Add functions
            for func in &schema.functions {
                items.push(CompletionItem {
                    label: func.name.clone(),
                    kind: "function".to_string(),
                    detail: Some(format!("Function in {}", func.schema)),
                });
            }

            // Add schemas
            for s in &schema.schemas {
                items.push(CompletionItem {
                    label: s.name.clone(),
                    kind: "schema".to_string(),
                    detail: None,
                });
            }
        }

        Ok(items)
    }

    /// Get hover information at a position in a file.
    ///
    /// Currently returns basic information from the schema.
    ///
    /// # Arguments
    /// * `path` - The virtual file path
    /// * `offset` - The byte offset in the file
    ///
    /// # Returns
    /// Hover text (markdown formatted), or None if no hover info is available.
    pub fn hover(&mut self, path: &str, offset: u32) -> Result<Option<String>, WasmError> {
        let content = self.fs.read(path)?;

        // Parse with tree-sitter to find node at position
        let tree = self
            .parser
            .parse(&content, None)
            .ok_or_else(|| WasmError::ParseError("Failed to parse SQL with tree-sitter".into()))?;

        // Find the node at the given offset
        let point = {
            let text_before = &content[..offset as usize];
            let row = text_before.matches('\n').count();
            let col = text_before.rfind('\n').map_or(offset as usize, |pos| offset as usize - pos - 1);
            tree_sitter::Point::new(row, col)
        };

        let node = tree.root_node().descendant_for_point_range(point, point);

        if let Some(node) = node {
            let node_text = &content[node.start_byte()..node.end_byte()];

            // Try to find matching schema info
            if let Some(schema) = &self.schema {
                // Check if it's a table name
                if let Some(table) = schema.find_tables(node_text, None).first() {
                    return Ok(Some(format!(
                        "**Table**: `{}.{}`\n\nColumns: {}",
                        table.schema,
                        table.name,
                        schema.columns
                            .iter()
                            .filter(|c| c.table_oid == table.id)
                            .map(|c| c.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }

                // Check if it's a function name
                if let Some(func) = schema.find_functions(node_text, None).first() {
                    return Ok(Some(format!(
                        "**Function**: `{}.{}`\n\nLanguage: {}",
                        func.schema,
                        func.name,
                        func.language
                    )));
                }
            }
        }

        Ok(None)
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
