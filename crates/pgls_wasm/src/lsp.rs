//! LSP JSON-RPC message handler for WASM.
//!
//! This module provides a synchronous LSP message handler that can process
//! JSON-RPC messages and return responses. It's designed to work with
//! `monaco-languageclient` via web workers.
//!
//! # Architecture
//!
//! ```text
//! Monaco Editor ←→ LanguageClient ←→ BrowserMessageReader/Writer ←→ Web Worker ←→ WASM
//! ```
//!
//! The `handle_message` function returns an array of JSON-RPC messages (response + notifications).
//! The web worker JavaScript iterates over this array and sends each message separately
//! via `postMessage`, which is what `BrowserMessageReader` expects.

use std::collections::HashMap;
use std::str::FromStr;

use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Workspace;

/// JSON-RPC request/notification structure
#[derive(Debug, Deserialize)]
struct JsonRpcMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// JSON-RPC notification structure (server → client)
#[derive(Debug, Serialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

/// Document state stored in the handler
struct DocumentState {
    content: String,
    version: i32,
}

/// LSP handler for WASM.
///
/// This handler processes LSP JSON-RPC messages synchronously and returns
/// an array of response/notification messages. It manages its own workspace
/// internally.
///
/// Use the `pgls/setSchema` notification to set the database schema:
/// ```json
/// {"jsonrpc":"2.0","method":"pgls/setSchema","params":{"schema":"..."}}
/// ```
pub struct LspHandler {
    workspace: Workspace,
    documents: HashMap<String, DocumentState>,
    initialized: bool,
    /// Whether the client accepts edits that name the document version.
    versioned_edits: bool,
}

impl Default for LspHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LspHandler {
    /// Create a new LSP handler with its own workspace.
    pub fn new() -> Self {
        Self {
            workspace: Workspace::new(),
            documents: HashMap::new(),
            initialized: false,
            versioned_edits: false,
        }
    }

    /// Handle an incoming LSP JSON-RPC message.
    ///
    /// Returns a JSON array of outgoing messages (response + notifications).
    /// For requests (with id), the array contains the response.
    /// For notifications (without id), the array may contain triggered notifications
    /// like `textDocument/publishDiagnostics`.
    pub fn handle_message(&mut self, message: &str) -> String {
        let mut outgoing: Vec<Value> = vec![];

        let req: JsonRpcMessage = match serde_json::from_str(message) {
            Ok(r) => r,
            Err(e) => {
                let error_response =
                    self.error_response(Value::Null, -32700, &format!("Parse error: {e}"));
                return serde_json::to_string(&vec![error_response]).unwrap();
            }
        };

        if let Some(id) = req.id {
            // Request → Response
            let response = self.handle_request(&req.method, req.params, id);
            outgoing.push(serde_json::to_value(response).unwrap());
        } else {
            // Notification → may trigger notifications back
            let notifications = self.handle_notification(&req.method, req.params);
            for n in notifications {
                outgoing.push(serde_json::to_value(n).unwrap());
            }
        }

        serde_json::to_string(&outgoing).unwrap()
    }

    fn handle_request(
        &mut self,
        method: &str,
        params: Option<Value>,
        id: Value,
    ) -> JsonRpcResponse {
        match method {
            "initialize" => self.initialize(params, id),
            "shutdown" => self.shutdown(id),
            "textDocument/completion" => self.completion(params, id),
            "textDocument/hover" => self.hover(params, id),
            "textDocument/codeAction" => self.code_action(params, id),
            _ => self.error_response(id, -32601, &format!("Method not found: {method}")),
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Vec<JsonRpcNotification> {
        match method {
            "initialized" => {
                self.initialized = true;
                vec![]
            }
            "textDocument/didOpen" => self.did_open(params),
            "textDocument/didChange" => self.did_change(params),
            "textDocument/didClose" => self.did_close(params),
            "pgls/setSchema" => self.set_schema_notification(params),
            "pgls/clearSchema" => self.clear_schema_notification(),
            _ => vec![],
        }
    }

    // -------------------------------------------------------------------------
    // Request handlers
    // -------------------------------------------------------------------------

    fn initialize(&mut self, params: Option<Value>, id: Value) -> JsonRpcResponse {
        self.versioned_edits = params
            .and_then(|params| serde_json::from_value::<InitializeParams>(params).ok())
            .and_then(|params| params.capabilities.workspace)
            .and_then(|workspace| workspace.workspace_edit)
            .and_then(|workspace_edit| workspace_edit.document_changes)
            == Some(true);
        let capabilities = InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "pgls-wasm".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        };

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::to_value(capabilities).unwrap()),
            error: None,
        }
    }

    fn shutdown(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(Value::Null),
            error: None,
        }
    }

    fn completion(&mut self, params: Option<Value>, id: Value) -> JsonRpcResponse {
        let params: CompletionParams = match params.and_then(|p| serde_json::from_value(p).ok()) {
            Some(p) => p,
            None => {
                return self.error_response(id, -32602, "Invalid params for completion");
            }
        };

        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let content = match self.documents.get(&uri) {
            Some(doc) => &doc.content,
            None => {
                return self.error_response(id, -32002, &format!("Document not found: {uri}"));
            }
        };

        let offset = position_to_offset(content, position);
        let completions = self
            .workspace
            .complete(&uri, offset as u32)
            .unwrap_or_default();

        let lsp_completions: Vec<CompletionItem> = completions
            .into_iter()
            .map(|c| CompletionItem {
                label: c.label,
                kind: Some(match c.kind.as_str() {
                    "table" => CompletionItemKind::CLASS,
                    "column" => CompletionItemKind::FIELD,
                    "function" => CompletionItemKind::FUNCTION,
                    "schema" => CompletionItemKind::MODULE,
                    _ => CompletionItemKind::TEXT,
                }),
                detail: c.detail,
                ..Default::default()
            })
            .collect();

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::to_value(lsp_completions).unwrap()),
            error: None,
        }
    }

    fn hover(&mut self, params: Option<Value>, id: Value) -> JsonRpcResponse {
        let params: HoverParams = match params.and_then(|p| serde_json::from_value(p).ok()) {
            Some(p) => p,
            None => {
                return self.error_response(id, -32602, "Invalid params for hover");
            }
        };

        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let content = match self.documents.get(&uri) {
            Some(doc) => &doc.content,
            None => {
                return self.error_response(id, -32002, &format!("Document not found: {uri}"));
            }
        };

        let offset = position_to_offset(content, position);
        let hover_text = self.workspace.hover(&uri, offset as u32).ok().flatten();

        let result = hover_text.map(|text| {
            serde_json::to_value(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: text,
                }),
                range: None,
            })
            .unwrap()
        });

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result,
            error: None,
        }
    }

    /// The fixes and suppressions of the lint diagnostics at the start of the range, like the
    /// native language server, and the action that applies all safe fixes when `context.only`
    /// asks for `source.fixAll.pgls`. There are no commands, since there is no database.
    fn code_action(&mut self, params: Option<Value>, id: Value) -> JsonRpcResponse {
        let params: CodeActionParams = match params.and_then(|p| serde_json::from_value(p).ok()) {
            Some(p) => p,
            None => {
                return self.error_response(id, -32602, "Invalid params for codeAction");
            }
        };

        let uri = params.text_document.uri.to_string();
        let Some(doc) = self.documents.get(&uri) else {
            return self.error_response(id, -32002, &format!("Document not found: {uri}"));
        };

        let requested = params
            .context
            .only
            .as_deref()
            .filter(|kinds| !kinds.is_empty());
        let mut actions = Vec::new();
        let only_fix_all =
            requested.is_some_and(|kinds| kinds.iter().all(|kind| covers(kind, FIX_ALL_KIND)));
        if !only_fix_all {
            let offset = position_to_offset(&doc.content, params.range.start);
            let workspace_actions = self
                .workspace
                .code_actions(&uri, offset as u32)
                .unwrap_or_default();
            for action in workspace_actions {
                let pgls_workspace::features::code_actions::CodeActionKind::Edit(edit) =
                    action.kind
                else {
                    continue;
                };
                let edits = edit
                    .edits
                    .iter()
                    .map(|text_edit| TextEdit {
                        range: Range {
                            start: offset_to_position(
                                &doc.content,
                                usize::from(text_edit.range.start()),
                            ),
                            end: offset_to_position(
                                &doc.content,
                                usize::from(text_edit.range.end()),
                            ),
                        },
                        new_text: text_edit.new_text.clone(),
                    })
                    .collect();
                let diagnostics: Vec<Diagnostic> = params
                    .context
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        edit.rule.as_ref().is_some_and(|rule| {
                            diagnostic.code == Some(NumberOrString::String(rule.clone()))
                        })
                    })
                    .cloned()
                    .collect();
                let is_fix = edit.rule.is_some()
                    && !edit
                        .category
                        .matches(pgls_analyse::SUPPRESSION_ACTION_CATEGORY);
                actions.push(CodeAction {
                    title: action.title,
                    kind: Some(CodeActionKind::from(edit.category.to_str().into_owned())),
                    diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
                    edit: Some(self.workspace_edit(&params.text_document.uri, doc.version, edits)),
                    is_preferred: (is_fix
                        && edit.applicability == pgls_analyse::Applicability::Always)
                        .then_some(true),
                    ..Default::default()
                });
            }
        }

        let wants_fix_all =
            requested.is_some_and(|kinds| kinds.iter().any(|kind| covers(kind, FIX_ALL_KIND)));
        if wants_fix_all
            && let Ok(fixed) = self.workspace.fix_file(&uri, false)
            && fixed.code != doc.content
        {
            actions.push(CodeAction {
                title: "Apply all safe fixes".into(),
                kind: Some(CodeActionKind::from(FIX_ALL_KIND)),
                edit: Some(self.workspace_edit(
                    &params.text_document.uri,
                    doc.version,
                    vec![changed_range(&doc.content, &fixed.code)],
                )),
                ..Default::default()
            });
        }

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::to_value(actions).unwrap()),
            error: None,
        }
    }

    /// An edit of the document, with its version if the client accepts versioned edits.
    fn workspace_edit(&self, uri: &Uri, version: i32, edits: Vec<TextEdit>) -> WorkspaceEdit {
        if self.versioned_edits {
            WorkspaceEdit {
                document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version: Some(version),
                    },
                    edits: edits.into_iter().map(OneOf::Left).collect(),
                }])),
                ..Default::default()
            }
        } else {
            WorkspaceEdit {
                changes: Some(HashMap::from([(uri.clone(), edits)])),
                ..Default::default()
            }
        }
    }

    // -------------------------------------------------------------------------
    // Notification handlers
    // -------------------------------------------------------------------------

    fn did_open(&mut self, params: Option<Value>) -> Vec<JsonRpcNotification> {
        let params: DidOpenTextDocumentParams =
            match params.and_then(|p| serde_json::from_value(p).ok()) {
                Some(p) => p,
                None => return vec![],
            };

        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.clone(),
                version,
            },
        );
        self.workspace.insert_file(&uri, &content);

        self.publish_diagnostics(&uri)
    }

    fn did_change(&mut self, params: Option<Value>) -> Vec<JsonRpcNotification> {
        let params: DidChangeTextDocumentParams =
            match params.and_then(|p| serde_json::from_value(p).ok()) {
                Some(p) => p,
                None => return vec![],
            };

        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        // We only support full sync, so take the last change
        let content = match params.content_changes.into_iter().last() {
            Some(change) => change.text,
            None => return vec![],
        };

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.clone(),
                version,
            },
        );
        self.workspace.insert_file(&uri, &content);

        self.publish_diagnostics(&uri)
    }

    fn did_close(&mut self, params: Option<Value>) -> Vec<JsonRpcNotification> {
        let params: DidCloseTextDocumentParams =
            match params.and_then(|p| serde_json::from_value(p).ok()) {
                Some(p) => p,
                None => return vec![],
            };

        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
        self.workspace.remove_file(&uri);

        // Send empty diagnostics to clear them
        vec![JsonRpcNotification {
            jsonrpc: "2.0".into(),
            method: "textDocument/publishDiagnostics".into(),
            params: serde_json::to_value(PublishDiagnosticsParams {
                uri: params.text_document.uri,
                diagnostics: vec![],
                version: None,
            })
            .unwrap(),
        }]
    }

    fn set_schema_notification(&mut self, params: Option<Value>) -> Vec<JsonRpcNotification> {
        #[derive(Deserialize)]
        struct SetSchemaParams {
            schema: String,
        }

        let params: SetSchemaParams = match params.and_then(|p| serde_json::from_value(p).ok()) {
            Some(p) => p,
            None => return vec![],
        };

        if let Err(e) = self.workspace.set_schema(&params.schema) {
            // Log error but don't fail
            eprintln!("Failed to set schema: {e}");
        }

        // Re-publish diagnostics for all open documents
        let uris: Vec<String> = self.documents.keys().cloned().collect();
        uris.into_iter()
            .flat_map(|uri| self.publish_diagnostics(&uri))
            .collect()
    }

    fn clear_schema_notification(&mut self) -> Vec<JsonRpcNotification> {
        self.workspace.clear_schema();
        vec![]
    }

    // -------------------------------------------------------------------------
    // Helper methods
    // -------------------------------------------------------------------------

    fn publish_diagnostics(&self, uri: &str) -> Vec<JsonRpcNotification> {
        let content = match self.documents.get(uri) {
            Some(doc) => &doc.content,
            None => return vec![],
        };

        let diagnostics = self.workspace.lint(uri).unwrap_or_default();
        let lsp_diagnostics: Vec<Diagnostic> = diagnostics
            .into_iter()
            .map(|d| Diagnostic {
                range: Range {
                    start: offset_to_position(content, d.start as usize),
                    end: offset_to_position(content, d.end as usize),
                },
                severity: Some(match d.severity.as_str() {
                    "error" => DiagnosticSeverity::ERROR,
                    "warning" => DiagnosticSeverity::WARNING,
                    "hint" => DiagnosticSeverity::HINT,
                    _ => DiagnosticSeverity::INFORMATION,
                }),
                message: d.message,
                code: (!d.category.is_empty()).then_some(NumberOrString::String(d.category)),
                source: Some("pgls".into()),
                ..Default::default()
            })
            .collect();

        vec![JsonRpcNotification {
            jsonrpc: "2.0".into(),
            method: "textDocument/publishDiagnostics".into(),
            params: serde_json::to_value(PublishDiagnosticsParams {
                uri: Uri::from_str(uri)
                    .unwrap_or_else(|_| Uri::from_str("file:///unknown").unwrap()),
                diagnostics: lsp_diagnostics,
                version: None,
            })
            .unwrap(),
        }]
    }

    fn error_response(&self, id: Value, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// The kind of the action that applies all safe fixes, like in the native language server.
const FIX_ALL_KIND: &str = "source.fixAll.pgls";

/// Whether the requested code action kind includes `kind`, e.g. `source` includes
/// `source.fixAll.pgls`.
fn covers(requested: &CodeActionKind, kind: &str) -> bool {
    let requested = requested.as_str();
    kind == requested
        || kind
            .strip_prefix(requested)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// The edit that turns `old` into `new`, replacing only what changed, so that the editor keeps
/// the cursor and the folds.
fn changed_range(old: &str, new: &str) -> TextEdit {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((index, _), _)| index);
    let suffix = old[prefix..]
        .chars()
        .rev()
        .zip(new[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    TextEdit {
        range: Range {
            start: offset_to_position(old, prefix),
            end: offset_to_position(old, old.len() - suffix),
        },
        new_text: new[prefix..new.len() - suffix].to_string(),
    }
}

// =============================================================================
// Position conversion utilities
// =============================================================================

/// Convert LSP Position (line:character in UTF-16) to byte offset.
fn position_to_offset(content: &str, pos: Position) -> usize {
    let mut offset = 0;

    for (i, line) in content.lines().enumerate() {
        if i == pos.line as usize {
            // Convert UTF-16 character offset to byte offset
            let mut utf16_count = 0u32;
            for (byte_idx, c) in line.char_indices() {
                if utf16_count >= pos.character {
                    return offset + byte_idx;
                }
                utf16_count += c.len_utf16() as u32;
            }
            return offset + line.len();
        }
        offset += line.len() + 1; // +1 for newline
    }

    offset.min(content.len())
}

/// Convert byte offset to LSP Position (line:character in UTF-16).
fn offset_to_position(content: &str, offset: usize) -> Position {
    let offset = offset.min(content.len());
    let text = &content[..offset];

    let line = text.matches('\n').count() as u32;
    let line_start = text.rfind('\n').map_or(0, |p| p + 1);
    let col = text[line_start..].encode_utf16().count() as u32;

    Position::new(line, col)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler() -> LspHandler {
        LspHandler::new()
    }

    #[test]
    fn test_initialize() {
        let mut handler = test_handler();
        let response = handler.handle_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0]["result"]["capabilities"].is_object());
        assert!(
            msgs[0]["result"]["serverInfo"]["name"]
                .as_str()
                .unwrap()
                .contains("pgls")
        );
    }

    #[test]
    fn test_did_open_returns_diagnostics() {
        let mut handler = test_handler();

        // Initialize first
        handler.handle_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        handler.handle_message(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);

        // Open a document with valid SQL
        let response = handler.handle_message(
            r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{
                    "uri":"file:///test.sql",
                    "languageId":"sql",
                    "version":1,
                    "text":"SELECT 1;"
                }
            }}"#,
        );

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0]["method"].as_str().unwrap(),
            "textDocument/publishDiagnostics"
        );
        // Valid SQL should have no diagnostics
        assert!(
            msgs[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_did_open_with_invalid_sql() {
        let mut handler = test_handler();

        handler.handle_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        handler.handle_message(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);

        // Open a document with invalid SQL
        let response = handler.handle_message(
            r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{
                    "uri":"file:///test.sql",
                    "languageId":"sql",
                    "version":1,
                    "text":"SELEC 1;"
                }
            }}"#,
        );

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0]["method"].as_str().unwrap(),
            "textDocument/publishDiagnostics"
        );
        // Invalid SQL should have diagnostics
        assert!(
            !msgs[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_completion() {
        let mut handler = test_handler();

        handler.handle_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        handler.handle_message(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
        handler.handle_message(
            r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{
                    "uri":"file:///test.sql",
                    "languageId":"sql",
                    "version":1,
                    "text":"SELECT "
                }
            }}"#,
        );

        let response = handler.handle_message(
            r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{
                "textDocument":{"uri":"file:///test.sql"},
                "position":{"line":0,"character":7}
            }}"#,
        );

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0]["result"].is_array());
    }

    #[test]
    fn test_hover() {
        let mut handler = test_handler();

        handler.handle_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        handler.handle_message(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
        handler.handle_message(
            r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{
                    "uri":"file:///test.sql",
                    "languageId":"sql",
                    "version":1,
                    "text":"SELECT * FROM users;"
                }
            }}"#,
        );

        let response = handler.handle_message(
            r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{
                "textDocument":{"uri":"file:///test.sql"},
                "position":{"line":0,"character":14}
            }}"#,
        );

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        // Without schema, hover returns null
        assert!(msgs[0]["result"].is_null());
    }

    fn open(handler: &mut LspHandler, capabilities: &str, text: &str) {
        handler.handle_message(&format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"capabilities":{capabilities}}}}}"#
        ));
        handler.handle_message(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
        handler.handle_message(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///test.sql",
                        "languageId": "sql",
                        "version": 3,
                        "text": text,
                    }
                }
            })
            .to_string(),
        );
    }

    fn code_actions(handler: &mut LspHandler, only: Option<&[&str]>) -> Vec<CodeAction> {
        let response = handler.handle_message(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/codeAction",
                "params": {
                    "textDocument": { "uri": "file:///test.sql" },
                    "range": {
                        "start": { "line": 0, "character": 2 },
                        "end": { "line": 0, "character": 2 }
                    },
                    "context": { "diagnostics": [], "only": only }
                }
            })
            .to_string(),
        );
        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        serde_json::from_value(msgs[0]["result"].clone()).unwrap()
    }

    #[test]
    fn test_code_actions_fix_and_suppress() {
        let mut handler = test_handler();
        open(&mut handler, "{}", "refresh materialized view user_stats;");

        let actions = code_actions(&mut handler, None);
        let titles: Vec<_> = actions.iter().map(|action| action.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Add CONCURRENTLY",
                "Suppress rule requireConcurrentRefreshMatview for this statement"
            ]
        );
        // The fix is unsafe, since it needs a unique index on the view.
        assert_eq!(actions[0].is_preferred, None);
        let edits = &actions[0].edit.as_ref().unwrap().changes.as_ref().unwrap()
            [&Uri::from_str("file:///test.sql").unwrap()];
        assert_eq!(edits[0].new_text.to_lowercase(), " concurrently");
        assert_eq!(edits[0].range.start, Position::new(0, 25));
    }

    #[test]
    fn test_code_actions_use_versioned_edits() {
        let mut handler = test_handler();
        open(
            &mut handler,
            r#"{"workspace":{"workspaceEdit":{"documentChanges":true}}}"#,
            "refresh materialized view user_stats;",
        );

        let actions = code_actions(&mut handler, None);
        let Some(DocumentChanges::Edits(changes)) = actions[0]
            .edit
            .as_ref()
            .and_then(|edit| edit.document_changes.as_ref())
        else {
            panic!("expected versioned edits: {actions:#?}");
        };
        assert_eq!(changes[0].text_document.version, Some(3));
    }

    #[test]
    fn test_fix_all_code_action() {
        let mut handler = test_handler();
        let text = "reindex index my_index;";
        open(&mut handler, "{}", text);

        // Only the fix-all action is computed for `source.fixAll.pgls`. It applies what `fix_file`
        // applies with the safe fixes, and is left out if that changes nothing.
        let actions = code_actions(&mut handler, Some(&["source.fixAll.pgls"]));
        let fixed = handler
            .workspace
            .fix_file("file:///test.sql", false)
            .unwrap()
            .code;
        if fixed == text {
            assert!(actions.is_empty(), "{actions:#?}");
        } else {
            assert_eq!(actions.len(), 1, "{actions:#?}");
            assert_eq!(
                actions[0].kind,
                Some(CodeActionKind::from("source.fixAll.pgls"))
            );
            let edit = &actions[0].edit.as_ref().unwrap().changes.as_ref().unwrap()
                [&Uri::from_str("file:///test.sql").unwrap()][0];
            let mut applied = text.to_string();
            applied.replace_range(
                position_to_offset(text, edit.range.start)
                    ..position_to_offset(text, edit.range.end),
                &edit.new_text,
            );
            assert_eq!(applied, fixed);
        }

        // The quick fixes come without the fix-all action.
        let actions = code_actions(&mut handler, Some(&["quickfix"]));
        assert!(
            actions
                .iter()
                .all(|action| action.title != "Apply all safe fixes"),
            "{actions:#?}"
        );
    }

    #[test]
    fn test_fix_file_applies_unsafe_fixes() {
        let mut workspace = Workspace::new();
        workspace.insert_file("/test.sql", "refresh materialized view user_stats;");
        assert_eq!(
            workspace
                .fix_file("/test.sql", true)
                .unwrap()
                .code
                .to_lowercase(),
            "refresh materialized view concurrently user_stats;"
        );
        assert_eq!(
            workspace.fix_file("/test.sql", false).unwrap().code,
            "refresh materialized view user_stats;"
        );
    }

    #[test]
    fn test_shutdown() {
        let mut handler = test_handler();

        let response =
            handler.handle_message(r#"{"jsonrpc":"2.0","id":1,"method":"shutdown","params":{}}"#);

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0]["result"].is_null());
        assert!(msgs[0]["error"].is_null());
    }

    #[test]
    fn test_unknown_method() {
        let mut handler = test_handler();

        let response = handler
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"method":"unknown/method","params":{}}"#);

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0]["error"].is_object());
        assert_eq!(msgs[0]["error"]["code"].as_i64().unwrap(), -32601);
    }

    #[test]
    fn test_parse_error() {
        let mut handler = test_handler();

        let response = handler.handle_message("not valid json");

        let msgs: Vec<Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0]["error"].is_object());
        assert_eq!(msgs[0]["error"]["code"].as_i64().unwrap(), -32700);
    }

    #[test]
    fn test_position_conversion() {
        let content = "SELECT\n  1\n  FROM users;";

        // Line 0, char 0 -> offset 0
        assert_eq!(position_to_offset(content, Position::new(0, 0)), 0);

        // Line 0, char 6 -> offset 6 (end of "SELECT")
        assert_eq!(position_to_offset(content, Position::new(0, 6)), 6);

        // Line 1, char 2 -> offset 9 ("  1" starts at offset 7, char 2 = offset 9)
        assert_eq!(position_to_offset(content, Position::new(1, 2)), 9);

        // Offset 0 -> line 0, char 0
        assert_eq!(offset_to_position(content, 0), Position::new(0, 0));

        // Offset 9 -> line 1, char 2
        assert_eq!(offset_to_position(content, 9), Position::new(1, 2));
    }

    #[test]
    fn test_position_conversion_utf16() {
        // Test with multi-byte characters
        let content = "SELECT '日本語';";

        // The string has: SELECT ' (8 bytes) + 日本語 (9 bytes) + '; (2 bytes) = 19 bytes
        // In UTF-16: SELECT ' (8 units) + 日本語 (3 units) + '; (2 units) = 13 units

        // Position at the end of '日本語'
        // Byte offset of ';' is 17
        let pos = offset_to_position(content, 17);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 11); // 8 + 3 = 11 UTF-16 units
    }
}
