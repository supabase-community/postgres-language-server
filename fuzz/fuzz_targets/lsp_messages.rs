//! Messages with arbitrary parameters to the language server, through the in-process client
//! `pgls_lsp::testing::Server`: positions and ranges anywhere, versions that go back, unknown
//! documents, URIs that aren't any, and parameters of the wrong shape. Every request answers,
//! with a result or an error, and nothing panics: libFuzzer's panic hook aborts on any panic,
//! also on one the server catches.
//!
//! `crates/pgls_lsp/tests/simulation.rs` drives the server through valid sessions and checks
//! what it publishes; this target is about the parameters.

#![no_main]

use std::time::Duration;

use arbitrary::Arbitrary;
use futures::channel::mpsc::channel;
use libfuzzer_sys::fuzz_target;
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_configuration::{Merge, PartialConfiguration};
use pgls_fs::MemoryFileSystem;
use pgls_lsp::ServerFactory;
use pgls_lsp::testing::{Client, Server};
use pgls_workspace::DynRef;
use serde_json::{Map, Value, json};

/// The longest session; a crash needs only a few messages.
const MESSAGES_MAX: usize = 32;

/// How long a request may take. A request without an answer takes all of it, and fails.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// The notifications of the server a session buffers; it reads none of them.
const NOTIFICATIONS_BUFFERED_MAX: usize = 1024;

#[derive(Arbitrary, Debug)]
struct Session {
    text: String,
    messages: Vec<Message>,
}

#[derive(Arbitrary, Debug)]
struct Message {
    method: Method,
    document: Document,
    start: (u32, u32),
    end: (u32, u32),
    version: i32,
    text: String,
    /// Replaces the parameters, to check how the server reads parameters of the wrong shape.
    raw: Option<Json>,
}

#[derive(Arbitrary, Debug, Clone, Copy)]
enum Method {
    Completion,
    Hover,
    CodeAction,
    Formatting,
    RangeFormatting,
    InlayHint,
    SemanticTokensFull,
    SemanticTokensRange,
    ExecuteCommand,
    SetConfigurationOverrides,
    Unknown,
    DidOpen,
    DidChange,
    DidChangeWholeText,
    DidSave,
    DidClose,
    DidChangeConfiguration,
    DidChangeWatchedFiles,
    CancelRequest,
}

/// The document a message is about.
#[derive(Arbitrary, Debug, Clone, Copy)]
enum Document {
    /// The document the session opens first.
    First,
    Other,
    NotAFileUri,
    NotAUri,
}

/// JSON of any shape, with the keys of LSP's parameters.
#[derive(Arbitrary, Debug)]
enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(Key, Json)>),
}

#[derive(Arbitrary, Debug, Clone, Copy)]
enum Key {
    TextDocument,
    Uri,
    Position,
    Range,
    Start,
    End,
    Line,
    Character,
    Version,
    Text,
    ContentChanges,
    Context,
    Options,
    Command,
    Arguments,
    Settings,
    Changes,
    Id,
    Other,
}

fuzz_target!(|session: Session| {
    if session.messages.len() > MESSAGES_MAX {
        return;
    }
    // A runtime per session: dropping it waits for the work the session started on the blocking
    // thread pool, so a panic there is the session's.
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_stack_size(pgls_workspace::WORKER_STACK_SIZE_BYTES)
        .build()
        .expect("build the runtime")
        .block_on(session.run());
});

impl Session {
    async fn run(self) {
        let mut configuration = PartialConfiguration::init();
        configuration.merge_with(PartialConfiguration {
            db: Some(PartialDatabaseConfiguration {
                disable_connection: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        });
        let mut fs = MemoryFileSystem::default();
        fs.insert(
            Server::uri("postgres-language-server.jsonc")
                .to_file_path()
                .expect("a file URI")
                .into_owned(),
            serde_json::to_string(&configuration).expect("serialize the configuration"),
        );
        let (service, socket) = ServerFactory::default()
            .create_with_fs(None, DynRef::Owned(Box::new(fs)))
            .into_inner();
        let (stream, sink) = socket.split();
        let (sender, notifications) = channel(NOTIFICATIONS_BUFFERED_MAX);
        let client = tokio::spawn(Client::run(stream, sink, sender));
        let mut server = Server::with_timeout(service, REQUEST_TIMEOUT);
        server.initialize().await.expect("initialize");
        server.initialized().await.expect("initialized");
        server
            .load_configuration()
            .await
            .expect("load the configuration");
        server.open_document(&self.text).await.expect("open");

        for message in self.messages {
            let method = message.method.name();
            let params = message.params();
            if message.method.is_request() {
                let response = server
                    .send_request(method, "fuzz", params)
                    .await
                    .unwrap_or_else(|e| panic!("lsp_messages: {method} failed to send: {e:#}"));
                if let Err(e) = response.await {
                    panic!("lsp_messages: {method} didn't answer: {e:#}");
                }
            } else {
                server
                    .notify(method, params)
                    .await
                    .unwrap_or_else(|e| panic!("lsp_messages: {method} failed: {e:#}"));
            }
        }

        server
            .shutdown()
            .await
            .unwrap_or_else(|e| panic!("lsp_messages: shutdown failed: {e:#}"));
        client.abort();
        drop(notifications);
    }
}

impl Method {
    fn name(self) -> &'static str {
        match self {
            Method::Completion => "textDocument/completion",
            Method::Hover => "textDocument/hover",
            Method::CodeAction => "textDocument/codeAction",
            Method::Formatting => "textDocument/formatting",
            Method::RangeFormatting => "textDocument/rangeFormatting",
            Method::InlayHint => "textDocument/inlayHint",
            Method::SemanticTokensFull => "textDocument/semanticTokens/full",
            Method::SemanticTokensRange => "textDocument/semanticTokens/range",
            Method::ExecuteCommand => "workspace/executeCommand",
            Method::SetConfigurationOverrides => "pgls/set_configuration_overrides",
            Method::Unknown => "pgls/unknown",
            Method::DidOpen => "textDocument/didOpen",
            Method::DidChange | Method::DidChangeWholeText => "textDocument/didChange",
            Method::DidSave => "textDocument/didSave",
            Method::DidClose => "textDocument/didClose",
            Method::DidChangeConfiguration => "workspace/didChangeConfiguration",
            Method::DidChangeWatchedFiles => "workspace/didChangeWatchedFiles",
            Method::CancelRequest => "$/cancelRequest",
        }
    }

    fn is_request(self) -> bool {
        matches!(
            self,
            Method::Completion
                | Method::Hover
                | Method::CodeAction
                | Method::Formatting
                | Method::RangeFormatting
                | Method::InlayHint
                | Method::SemanticTokensFull
                | Method::SemanticTokensRange
                | Method::ExecuteCommand
                | Method::SetConfigurationOverrides
                | Method::Unknown
        )
    }
}

impl Message {
    /// The parameters: `raw`, or an object with the fields of every method, so that each reads
    /// the ones it knows.
    fn params(&self) -> Value {
        if let Some(raw) = &self.raw {
            return raw.value();
        }
        let uri = match self.document {
            Document::First => Server::uri("document.sql").to_string(),
            Document::Other => Server::uri("other.sql").to_string(),
            Document::NotAFileUri => "untitled:Untitled-1".to_owned(),
            Document::NotAUri => self.text.clone(),
        };
        let position =
            |(line, character): (u32, u32)| json!({"line": line, "character": character});
        let range = json!({"start": position(self.start), "end": position(self.end)});
        let change = match self.method {
            Method::DidChangeWholeText => json!({"text": self.text}),
            _ => json!({"range": range, "text": self.text}),
        };
        json!({
            "textDocument": {"uri": uri, "languageId": "sql", "version": self.version, "text": self.text},
            "position": position(self.start),
            "range": range,
            "context": {"diagnostics": [], "triggerKind": 1},
            "options": {"tabSize": 4, "insertSpaces": true},
            "contentChanges": [change],
            "text": self.text,
            "command": self.text,
            "arguments": [uri],
            "settings": {},
            "changes": [{"uri": uri, "type": 2}],
            "id": "fuzz",
        })
    }
}

impl Json {
    fn value(&self) -> Value {
        match self {
            Json::Null => Value::Null,
            Json::Bool(value) => Value::Bool(*value),
            Json::Int(value) => Value::from(*value),
            Json::Float(value) => Value::from(*value),
            Json::String(value) => Value::String(value.clone()),
            Json::Array(values) => Value::Array(values.iter().map(Json::value).collect()),
            Json::Object(entries) => Value::Object(
                entries
                    .iter()
                    .map(|(key, value)| (key.name().to_owned(), value.value()))
                    .collect::<Map<_, _>>(),
            ),
        }
    }
}

impl Key {
    fn name(self) -> &'static str {
        match self {
            Key::TextDocument => "textDocument",
            Key::Uri => "uri",
            Key::Position => "position",
            Key::Range => "range",
            Key::Start => "start",
            Key::End => "end",
            Key::Line => "line",
            Key::Character => "character",
            Key::Version => "version",
            Key::Text => "text",
            Key::ContentChanges => "contentChanges",
            Key::Context => "context",
            Key::Options => "options",
            Key::Command => "command",
            Key::Arguments => "arguments",
            Key::Settings => "settings",
            Key::Changes => "changes",
            Key::Id => "id",
            Key::Other => "other",
        }
    }
}
