//! A simulated editor that drives the language server through random sessions.
//!
//! A session is a sequence of the editor's actions: it opens, edits, replaces and closes
//! documents, sends requests and cancels some of them, waits, and changes the configuration.
//! proptest-state-machine generates the sessions and shrinks a failing one to a short one.
//!
//! The server runs on a current-thread runtime whose clock is paused, the workspace runs its async
//! work on the same runtime, and the work that blocks runs inline ([`Blocking::Inline`]). So a
//! session gives one interleaving of the server's tasks, the debounces fire at their virtual time,
//! and a failure replays. There is no database: the connection is disabled.
//!
//! After every action, the simulation checks that:
//! - every request answers within [`REQUEST_TIMEOUT`] of virtual time;
//! - nothing panics;
//! - the ranges and positions of every response, and of the published diagnostics, are valid in
//!   the client's text of the version they are for, as LSP splits lines;
//! - the diagnostics of a document are never published for an older version than earlier ones,
//!   nor for a closed document.
//!
//! At the end of a session, the last diagnostics of every open document are for its current
//! version, and equal those of a fresh server that opens the final texts.
//!
//! `PROPTEST_CASES` sets the number of sessions, [`SESSIONS_DEFAULT`] by default.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Once};
use std::time::Duration;

use anyhow::Result;
use futures::channel::mpsc::{Receiver, channel};
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_configuration::{Merge, PartialConfiguration};
use pgls_fs::MemoryFileSystem;
use pgls_lsp::testing::{Client, Server, ServerNotification};
use pgls_lsp::{Blocking, ServerFactory};
use pgls_workspace::DynRef;
use pgls_workspace::workspace::server::WorkspaceServer;
use proptest::prelude::*;
use proptest::strategy::Union;
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest, prop_state_machine};
use serde_json::{Value, from_value, json};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tower_lsp_server::jsonrpc::ErrorCode;
use tower_lsp_server::ls_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, CompletionParams, CompletionResponse,
    CompletionTextEdit, Diagnostic, DocumentFormattingParams, FormattingOptions, Hover,
    HoverParams, InlayHint, InlayHintParams, PartialResultParams, Position,
    PublishDiagnosticsParams, Range, SemanticTokensParams, SemanticTokensRangeParams,
    SemanticTokensRangeResult, SemanticTokensResult, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentPositionParams, TextEdit, Uri, WorkDoneProgressParams,
};

/// The number of sessions without `PROPTEST_CASES`.
const SESSIONS_DEFAULT: u32 = 64;

/// The number of documents a session uses.
const DOCUMENTS: usize = 3;

/// How long a request may take, in virtual time. Only a request that waits for something that
/// never happens takes any.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a session may take in real time. A session that waits for something outside its
/// runtime never ends, since its clock is paused, so the [`Watchdog`] aborts it.
const SESSION_WALL_TIME_MAX: Duration = Duration::from_secs(120);

/// How long the end of a session waits for the debounced work: the diagnostics and a
/// configuration reload.
const SETTLE_TIMEOUT: Duration = Duration::from_secs(5);

/// The statements the texts are made of: valid and broken ones, multi-byte characters, `\r\n`.
const STATEMENTS: &[&str] = &[
    "select 1;\n",
    "select a, b from t where a = 1;\n",
    "create table t (id int primary key, name text);\n",
    "alter table t add column c text;\n",
    "insert into t values (1, 'ü€');\n",
    "-- a comment with 😀\n",
    "select * from t;\r\n",
    "selec 1;\n",
    "create function f() returns int language plpgsql as $$ begin return 1; end $$;\n",
    "select 'unterminated\n",
    "/* a block\n comment */\n",
    "update t set name = 'x' where id = 1;\n",
    "drop table t;\n",
    "with x as (select 1) select * from x;\n",
    "create index on t (name);\n",
    "select\n  1\n;\n",
];

/// What edits insert: pieces of statements, multi-byte characters, line breaks.
const SNIPPETS: &[&str] = &[
    "", " ", ";", "\n", "\r\n", "(", ")", "'", "$$", "é", "😀", "select", " from t", "1", ",",
    "--", "/*", "*/", "table",
];

/// A text as the client sees it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ClientText(String);

impl ClientText {
    /// The byte offset of the character at `chars`, or of the end. An offset between `\r` and
    /// `\n` moves before the `\r`, since no LSP position is between them.
    fn byte_offset(&self, chars: usize) -> usize {
        let byte = self
            .0
            .char_indices()
            .nth(chars)
            .map_or(self.0.len(), |(byte, _)| byte);
        let bytes = self.0.as_bytes();
        if byte > 0 && bytes[byte - 1] == b'\r' && bytes.get(byte) == Some(&b'\n') {
            byte - 1
        } else {
            byte
        }
    }

    fn char_count(&self) -> usize {
        self.0.chars().count()
    }

    /// The lines as LSP splits them, at `\r\n`, `\n` and `\r`, without the line breaks.
    fn lines(&self) -> Vec<&str> {
        let mut lines = Vec::new();
        let mut rest = self.0.as_str();
        while let Some(end) = rest.find(['\r', '\n']) {
            lines.push(&rest[..end]);
            let break_len = if rest[end..].starts_with("\r\n") {
                2
            } else {
                1
            };
            rest = &rest[end + break_len..];
        }
        lines.push(rest);
        lines
    }

    /// The UTF-16 position of the byte at `byte`.
    fn position(&self, byte: usize) -> Position {
        let before = ClientText(self.0[..byte].to_string());
        let lines = before.lines();
        let last = lines.last().copied().unwrap_or_default();
        Position::new(
            u32::try_from(lines.len() - 1).unwrap(),
            u32::try_from(last.encode_utf16().count()).unwrap(),
        )
    }

    /// Whether `position` is in the text and not inside a surrogate pair.
    fn is_valid(&self, position: Position) -> bool {
        let lines = self.lines();
        let Some(line) = lines.get(position.line as usize) else {
            return false;
        };
        let units: Vec<u16> = line.encode_utf16().collect();
        let character = position.character as usize;
        character <= units.len()
            && (character == 0
                || character == units.len()
                || !(0xD800..0xDC00).contains(&units[character - 1]))
    }

    /// Panics unless `range` is a valid range in the text.
    fn check(&self, range: Range, what: &str) {
        assert!(
            self.is_valid(range.start)
                && self.is_valid(range.end)
                && (range.start.line, range.start.character)
                    <= (range.end.line, range.end.character),
            "{what}: {range:?} is not a valid range in {:?}",
            self.0
        );
    }

    /// Replaces the characters `start..end` with `insert`, and returns the range it replaced and
    /// the new text.
    fn edit(&self, start: usize, end: usize, insert: &str) -> (Range, ClientText) {
        let (start, end) = (self.byte_offset(start), self.byte_offset(end));
        let range = Range::new(self.position(start), self.position(end));
        let mut text = self.0.clone();
        text.replace_range(start..end, insert);
        (range, ClientText(text))
    }
}

/// A request of the client.
#[derive(Clone, Copy, Debug)]
enum RequestKind {
    Hover,
    Completion,
    CodeAction,
    SemanticTokens,
    SemanticTokensRange,
    InlayHints,
    Formatting,
}

impl RequestKind {
    const ALL: [RequestKind; 7] = [
        Self::Hover,
        Self::Completion,
        Self::CodeAction,
        Self::SemanticTokens,
        Self::SemanticTokensRange,
        Self::InlayHints,
        Self::Formatting,
    ];

    fn method(self) -> &'static str {
        match self {
            Self::Hover => "textDocument/hover",
            Self::Completion => "textDocument/completion",
            Self::CodeAction => "textDocument/codeAction",
            Self::SemanticTokens => "textDocument/semanticTokens/full",
            Self::SemanticTokensRange => "textDocument/semanticTokens/range",
            Self::InlayHints => "textDocument/inlayHint",
            Self::Formatting => "textDocument/formatting",
        }
    }

    /// The parameters of the request on `uri`, at `position`, in a text that ends at `end`.
    fn params(self, uri: Uri, position: Position, end: Position) -> Value {
        let text_document = TextDocumentIdentifier { uri };
        let whole = Range::new(Position::new(0, 0), end);
        let at = TextDocumentPositionParams {
            text_document: text_document.clone(),
            position,
        };
        let work_done_progress_params = WorkDoneProgressParams::default();
        let partial_result_params = PartialResultParams::default();
        match self {
            Self::Hover => json!(HoverParams {
                text_document_position_params: at,
                work_done_progress_params,
            }),
            Self::Completion => json!(CompletionParams {
                text_document_position: at,
                work_done_progress_params,
                partial_result_params,
                context: None,
            }),
            Self::CodeAction => json!(CodeActionParams {
                text_document,
                range: Range::new(position, position),
                context: CodeActionContext::default(),
                work_done_progress_params,
                partial_result_params,
            }),
            Self::SemanticTokens => json!(SemanticTokensParams {
                text_document,
                work_done_progress_params,
                partial_result_params,
            }),
            Self::SemanticTokensRange => json!(SemanticTokensRangeParams {
                text_document,
                range: Range::new(Position::new(0, 0), position),
                work_done_progress_params,
                partial_result_params,
            }),
            Self::InlayHints => json!(InlayHintParams {
                text_document,
                range: whole,
                work_done_progress_params,
            }),
            Self::Formatting => json!(DocumentFormattingParams {
                text_document,
                options: FormattingOptions {
                    tab_size: 2,
                    insert_spaces: true,
                    ..FormattingOptions::default()
                },
                work_done_progress_params,
            }),
        }
    }

    /// Panics unless every range and position in `result` is valid in `text`.
    fn check(self, result: Value, text: &ClientText, uri: &Uri) {
        if result.is_null() {
            return;
        }
        let what = self.method();
        match self {
            Self::Hover => {
                let hover: Hover = from_value(result).unwrap();
                if let Some(range) = hover.range {
                    text.check(range, what);
                }
            }
            Self::Completion => {
                let items = match from_value(result).unwrap() {
                    CompletionResponse::Array(items) => items,
                    CompletionResponse::List(list) => list.items,
                };
                for item in items {
                    match item.text_edit {
                        Some(CompletionTextEdit::Edit(edit)) => text.check(edit.range, what),
                        Some(CompletionTextEdit::InsertAndReplace(edit)) => {
                            text.check(edit.insert, what);
                            text.check(edit.replace, what);
                        }
                        None => {}
                    }
                    for edit in item.additional_text_edits.unwrap_or_default() {
                        text.check(edit.range, what);
                    }
                }
            }
            Self::CodeAction => {
                let actions: Vec<CodeActionOrCommand> = from_value(result).unwrap();
                for action in actions {
                    let CodeActionOrCommand::CodeAction(action) = action else {
                        continue;
                    };
                    let changes = action.edit.and_then(|edit| edit.changes);
                    for (edited, edits) in changes.unwrap_or_default() {
                        assert_eq!(&edited, uri, "{what}: an edit of another document");
                        for edit in edits {
                            text.check(edit.range, what);
                        }
                    }
                }
            }
            Self::SemanticTokens => {
                let SemanticTokensResult::Tokens(tokens) = from_value(result).unwrap() else {
                    panic!("{what}: the server answered with partial tokens");
                };
                Self::check_tokens(&tokens.data, text);
            }
            Self::SemanticTokensRange => {
                let SemanticTokensRangeResult::Tokens(tokens) = from_value(result).unwrap() else {
                    panic!("{what}: the server answered with partial tokens");
                };
                Self::check_tokens(&tokens.data, text);
            }
            Self::InlayHints => {
                let hints: Vec<InlayHint> = from_value(result).unwrap();
                for hint in hints {
                    text.check(Range::new(hint.position, hint.position), what);
                }
            }
            Self::Formatting => {
                let edits: Vec<TextEdit> = from_value(result).unwrap();
                for edit in edits {
                    text.check(edit.range, what);
                }
            }
        }
    }

    /// Panics unless every token is on one line of `text`.
    fn check_tokens(tokens: &[tower_lsp_server::ls_types::SemanticToken], text: &ClientText) {
        let (mut line, mut start) = (0, 0);
        for token in tokens {
            if token.delta_line > 0 {
                line += token.delta_line;
                start = token.delta_start;
            } else {
                start += token.delta_start;
            }
            text.check(
                Range::new(
                    Position::new(line, start),
                    Position::new(line, start + token.length),
                ),
                "semantic token",
            );
        }
    }
}

/// What the editor does.
#[derive(Clone, Debug)]
enum Action {
    Open {
        document: usize,
        text: String,
    },
    /// Replaces the characters `start..end`.
    Edit {
        document: usize,
        start: usize,
        end: usize,
        text: String,
    },
    Replace {
        document: usize,
        text: String,
    },
    /// An edit at a position after the end of the text, which the client then replaces with the
    /// text it has.
    EditOutOfRange {
        document: usize,
        line: u32,
        character: u32,
    },
    Close {
        document: usize,
    },
    /// A request at the character `at`, or after the end of the text.
    Request {
        document: usize,
        request: RequestKind,
        at: usize,
        out_of_range: bool,
        cancel: bool,
    },
    Wait {
        millis: u64,
    },
    /// Sets the configuration overrides of the client.
    SetLinter {
        enabled: bool,
    },
    /// Tells the server that the configuration file changed.
    ReloadConfiguration,
}

impl Action {
    fn text() -> BoxedStrategy<String> {
        prop::collection::vec(prop::sample::select(STATEMENTS), 0..5)
            .prop_map(|statements| statements.concat())
            .boxed()
    }

    fn snippet() -> BoxedStrategy<String> {
        prop_oneof![
            3 => prop::sample::select(SNIPPETS).prop_map(str::to_string),
            1 => prop::sample::select(STATEMENTS).prop_map(str::to_string),
        ]
        .boxed()
    }
}

/// The editor's view of a session: the documents it has open, and its configuration.
#[derive(Clone, Debug, Default)]
struct Editor {
    documents: [Option<ClientText>; DOCUMENTS],
    linter: Option<bool>,
}

impl ReferenceStateMachine for Editor {
    type State = Self;
    type Transition = Action;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(Self::default()).boxed()
    }

    fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
        let mut actions: Vec<(u32, BoxedStrategy<Action>)> = vec![
            (
                2,
                (1..400u64)
                    .prop_map(|millis| Action::Wait { millis })
                    .boxed(),
            ),
            (
                1,
                any::<bool>()
                    .prop_map(|enabled| Action::SetLinter { enabled })
                    .boxed(),
            ),
            (1, Just(Action::ReloadConfiguration).boxed()),
        ];
        for (document, text) in state.documents.iter().enumerate() {
            let Some(text) = text else {
                actions.push((
                    3,
                    Action::text()
                        .prop_map(move |text| Action::Open { document, text })
                        .boxed(),
                ));
                continue;
            };
            let chars = text.char_count();
            actions.push((
                8,
                (0..=chars, 0..=chars, Action::snippet())
                    .prop_map(move |(a, b, text)| Action::Edit {
                        document,
                        start: a.min(b),
                        end: a.max(b),
                        text,
                    })
                    .boxed(),
            ));
            actions.push((
                1,
                Action::text()
                    .prop_map(move |text| Action::Replace { document, text })
                    .boxed(),
            ));
            actions.push((
                1,
                (0..4u32, 0..200u32)
                    .prop_map(move |(line, character)| Action::EditOutOfRange {
                        document,
                        line,
                        character,
                    })
                    .boxed(),
            ));
            actions.push((1, Just(Action::Close { document }).boxed()));
            actions.push((
                6,
                (
                    prop::sample::select(&RequestKind::ALL[..]),
                    0..=chars,
                    prop::bool::weighted(0.1),
                    prop::bool::weighted(0.2),
                )
                    .prop_map(move |(request, at, out_of_range, cancel)| Action::Request {
                        document,
                        request,
                        at,
                        out_of_range,
                        cancel,
                    })
                    .boxed(),
            ));
        }
        Union::new_weighted(actions).boxed()
    }

    fn preconditions(state: &Self::State, action: &Self::Transition) -> bool {
        // Shrinking drops actions, so an action must still fit the state it now follows.
        match action {
            Action::Open { document, .. } => state.documents[*document].is_none(),
            Action::Edit {
                document,
                start,
                end,
                ..
            } => state.documents[*document]
                .as_ref()
                .is_some_and(|text| start <= end && *end <= text.char_count()),
            Action::Request { document, at, .. } => state.documents[*document]
                .as_ref()
                .is_some_and(|text| *at <= text.char_count()),
            Action::Replace { document, .. }
            | Action::EditOutOfRange { document, .. }
            | Action::Close { document } => state.documents[*document].is_some(),
            Action::Wait { .. } | Action::SetLinter { .. } | Action::ReloadConfiguration => true,
        }
    }

    fn apply(mut state: Self::State, action: &Self::Transition) -> Self::State {
        match action {
            Action::Open { document, text } | Action::Replace { document, text } => {
                state.documents[*document] = Some(ClientText(text.clone()));
            }
            Action::Edit {
                document,
                start,
                end,
                text,
            } => {
                let document = &mut state.documents[*document];
                let edited = document.as_ref().unwrap().edit(*start, *end, text).1;
                *document = Some(edited);
            }
            Action::Close { document } => state.documents[*document] = None,
            Action::SetLinter { enabled } => state.linter = Some(*enabled),
            Action::EditOutOfRange { .. }
            | Action::Request { .. }
            | Action::Wait { .. }
            | Action::ReloadConfiguration => {}
        }
        state
    }
}

/// The panics of this test binary so far, see [`Simulation::count_panics`].
static PANICS: AtomicUsize = AtomicUsize::new(0);

/// An open document, as the client sees it.
struct ClientDocument {
    /// The text of every version since the document was opened, by version. `None` if the
    /// client doesn't know it: after an edit out of range.
    texts: Vec<Option<ClientText>>,
    /// The newest version the server published diagnostics for, and the diagnostics.
    published: Option<(i32, Vec<Diagnostic>)>,
}

impl ClientDocument {
    fn version(&self) -> i32 {
        i32::try_from(self.texts.len() - 1).unwrap()
    }

    fn text(&self) -> &ClientText {
        self.texts.last().unwrap().as_ref().unwrap()
    }

    fn push(&mut self, text: Option<ClientText>) -> i32 {
        self.texts.push(text);
        self.version()
    }
}

/// A language server and the client that talks to it.
struct Connection {
    server: Server,
    notifications: Receiver<ServerNotification>,
    client: JoinHandle<Result<()>>,
}

impl Connection {
    /// Starts a server on the current runtime, with the database connection disabled and the
    /// linter `enabled`, if set.
    async fn start(linter: Option<bool>) -> Self {
        let workspace = WorkspaceServer::with_runtime(tokio::runtime::Handle::current());
        let factory = ServerFactory::default()
            .with_workspace(Arc::new(workspace))
            .with_blocking(Blocking::Inline);
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
                .unwrap()
                .into_owned(),
            serde_json::to_string_pretty(&configuration).unwrap(),
        );
        let (service, socket) = factory
            .create_with_fs(None, DynRef::Owned(Box::new(fs)))
            .into_inner();
        let (stream, sink) = socket.split();
        // The simulation reads the notifications after every action, which can publish the
        // diagnostics of every document.
        let (sender, notifications) = channel(64);
        let client = tokio::spawn(Client::run(stream, sink, sender));
        let mut server = Server::with_timeout(service, REQUEST_TIMEOUT);
        server.initialize().await.unwrap();
        server.initialized().await.unwrap();
        server.load_configuration().await.unwrap();
        if let Some(enabled) = linter {
            server
                .set_configuration_overrides(
                    json!({ "overrides": { "linter": { "enabled": enabled } } }),
                )
                .await
                .unwrap();
        }
        Self {
            server,
            notifications,
            client,
        }
    }

    /// Lets the server finish what it can without the clock moving on, and a millisecond more.
    async fn settle() {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    async fn stop(mut self) {
        self.server.shutdown().await.unwrap();
        self.client.abort();
    }
}

/// The system under test: a session and the runtime it runs on.
struct Simulation {
    runtime: Runtime,
    session: Session,
    /// The panics before the session started.
    panics: usize,
    _watchdog: Watchdog,
}

/// Aborts the test binary when a session takes longer than [`SESSION_WALL_TIME_MAX`], until it
/// is dropped.
struct Watchdog {
    /// Tells the watchdog that the session ended, when it's dropped.
    _alive: mpsc::Sender<()>,
}

impl Watchdog {
    fn start() -> Self {
        let (alive, session) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            if let Err(RecvTimeoutError::Timeout) = session.recv_timeout(SESSION_WALL_TIME_MAX) {
                eprintln!(
                    "a session took longer than {SESSION_WALL_TIME_MAX:?}: it waits for something outside its runtime"
                );
                std::process::abort();
            }
        });
        Self { _alive: alive }
    }
}

impl Simulation {
    /// Counts the panics of the test binary in [`PANICS`], including those the server catches.
    fn count_panics() {
        static HOOK: Once = Once::new();
        HOOK.call_once(|| {
            let default = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                PANICS.fetch_add(1, Ordering::SeqCst);
                default(info);
            }));
        });
    }
}

/// The server and the client's documents.
struct Session {
    connection: Connection,
    uris: [Uri; DOCUMENTS],
    documents: [Option<ClientDocument>; DOCUMENTS],
    linter: Option<bool>,
}

impl Session {
    fn document(&self, document: usize) -> &ClientDocument {
        self.documents[document].as_ref().unwrap()
    }

    fn document_mut(&mut self, document: usize) -> &mut ClientDocument {
        self.documents[document].as_mut().unwrap()
    }

    async fn act(&mut self, action: Action) {
        match action {
            Action::Open { document, text } => {
                self.connection
                    .server
                    .open_named_document(&text, self.uris[document].clone(), "sql")
                    .await
                    .unwrap();
                self.documents[document] = Some(ClientDocument {
                    texts: vec![Some(ClientText(text))],
                    published: None,
                });
            }
            Action::Edit {
                document,
                start,
                end,
                text,
            } => {
                let (range, edited) = self.document(document).text().edit(start, end, &text);
                let version = self.document_mut(document).push(Some(edited));
                let change = TextDocumentContentChangeEvent {
                    range: Some(range),
                    range_length: None,
                    text,
                };
                self.connection
                    .server
                    .change_named_document(self.uris[document].clone(), version, vec![change])
                    .await
                    .unwrap();
            }
            Action::Replace { document, text } => {
                let version = self
                    .document_mut(document)
                    .push(Some(ClientText(text.clone())));
                self.connection
                    .server
                    .change_named_document(
                        self.uris[document].clone(),
                        version,
                        vec![TextDocumentContentChangeEvent {
                            range: None,
                            range_length: None,
                            text,
                        }],
                    )
                    .await
                    .unwrap();
            }
            Action::EditOutOfRange {
                document,
                line,
                character,
            } => {
                let text = self.document(document).text().clone();
                let lines = u32::try_from(text.lines().len()).unwrap();
                let start = Position::new(lines + line, character);
                let out_of_range = TextDocumentContentChangeEvent {
                    range: Some(Range::new(
                        start,
                        Position::new(start.line, start.character + 1),
                    )),
                    range_length: None,
                    text: "x".to_string(),
                };
                let uri = self.uris[document].clone();
                let document = self.documents[document].as_mut().unwrap();
                let version = document.push(None);
                self.connection
                    .server
                    .change_named_document(uri.clone(), version, vec![out_of_range])
                    .await
                    .unwrap();
                let version = document.push(Some(text.clone()));
                self.connection
                    .server
                    .change_named_document(
                        uri,
                        version,
                        vec![TextDocumentContentChangeEvent {
                            range: None,
                            range_length: None,
                            text: text.0,
                        }],
                    )
                    .await
                    .unwrap();
            }
            Action::Close { document } => {
                self.connection
                    .server
                    .close_named_document(self.uris[document].clone())
                    .await
                    .unwrap();
                self.documents[document] = None;
            }
            Action::Request {
                document,
                request,
                at,
                out_of_range,
                cancel,
            } => {
                let uri = self.uris[document].clone();
                let text = self.document(document).text().clone();
                let end = text.position(text.0.len());
                let position = if out_of_range {
                    Position::new(end.line + 2, 3)
                } else {
                    text.position(text.byte_offset(at))
                };
                let id = request.method();
                let response = self
                    .connection
                    .server
                    .send_request(id, id, request.params(uri.clone(), position, end))
                    .await
                    .unwrap();
                if cancel {
                    self.connection.server.cancel_request(id).await.unwrap();
                }
                let response = response
                    .await
                    .unwrap_or_else(|err| panic!("{id} at {position:?}: {err:#}"))
                    .unwrap_or_else(|| panic!("{id} got no response"));
                match response.into_parts().1 {
                    Ok(result) => request.check(result, &text, &uri),
                    Err(error) => assert!(
                        out_of_range || (cancel && error.code == ErrorCode::RequestCancelled),
                        "{id} at {position:?} failed: {error:?}"
                    ),
                }
            }
            Action::Wait { millis } => {
                tokio::time::sleep(Duration::from_millis(millis)).await;
            }
            Action::SetLinter { enabled } => {
                self.connection
                    .server
                    .set_configuration_overrides(
                        json!({ "overrides": { "linter": { "enabled": enabled } } }),
                    )
                    .await
                    .unwrap();
                self.linter = Some(enabled);
            }
            Action::ReloadConfiguration => {
                self.connection
                    .server
                    .change_watched_configuration_file()
                    .await
                    .unwrap();
            }
        }
        Connection::settle().await;
        self.read_notifications();
    }

    /// Checks the diagnostics the server published since the last call.
    fn read_notifications(&mut self) {
        while let Ok(Some(notification)) = self.connection.notifications.try_next() {
            let ServerNotification::PublishDiagnostics(params) = notification else {
                continue;
            };
            self.check_published(params);
        }
    }

    fn check_published(&mut self, params: PublishDiagnosticsParams) {
        let index = self
            .uris
            .iter()
            .position(|uri| *uri == params.uri)
            .unwrap_or_else(|| panic!("diagnostics of an unknown document: {:?}", params.uri));
        let Some(document) = self.documents[index].as_mut() else {
            assert!(
                params.diagnostics.is_empty(),
                "diagnostics of the closed document {index}: {params:?}"
            );
            return;
        };
        let version = params
            .version
            .unwrap_or_else(|| panic!("diagnostics without a version: {params:?}"));
        if let Some((published, _)) = &document.published {
            assert!(
                *published <= version,
                "the diagnostics of version {version} of document {index} came after those of version {published}"
            );
        }
        let text = usize::try_from(version)
            .ok()
            .and_then(|version| document.texts.get(version))
            .unwrap_or_else(|| {
                panic!("diagnostics of version {version} of document {index}, which it never had")
            });
        if let Some(text) = text {
            for diagnostic in &params.diagnostics {
                text.check(
                    diagnostic.range,
                    &format!("diagnostic {:?}", diagnostic.message),
                );
            }
        }
        document.published = Some((version, params.diagnostics));
    }

    /// The diagnostics in an order that doesn't depend on the server.
    fn sorted(diagnostics: &[Diagnostic]) -> Vec<String> {
        let mut sorted: Vec<String> = diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "{:?} {:?} {:?} {}",
                    diagnostic.range, diagnostic.severity, diagnostic.code, diagnostic.message
                )
            })
            .collect();
        sorted.sort();
        sorted
    }

    /// Waits for the debounced work, and checks that the last diagnostics of every open document
    /// are for its current version, and equal those of a fresh server that opens the final texts.
    async fn finish(&mut self) {
        tokio::time::sleep(SETTLE_TIMEOUT).await;
        self.read_notifications();

        let mut fresh = Connection::start(self.linter).await;
        for (index, document) in self.documents.iter().enumerate() {
            let Some(document) = document else {
                continue;
            };
            fresh
                .server
                .open_named_document(&document.text().0, self.uris[index].clone(), "sql")
                .await
                .unwrap();
        }
        tokio::time::sleep(SETTLE_TIMEOUT).await;
        let mut expected: [Option<Vec<Diagnostic>>; DOCUMENTS] = Default::default();
        while let Ok(Some(notification)) = fresh.notifications.try_next() {
            if let ServerNotification::PublishDiagnostics(params) = notification
                && let Some(index) = self.uris.iter().position(|uri| *uri == params.uri)
            {
                expected[index] = Some(params.diagnostics);
            }
        }
        fresh.stop().await;

        for (index, document) in self.documents.iter().enumerate() {
            let Some(document) = document else {
                continue;
            };
            let version = document.version();
            let Some((published, diagnostics)) = &document.published else {
                panic!("no diagnostics of document {index}");
            };
            assert_eq!(
                *published, version,
                "the last diagnostics of document {index} are for version {published}, not {version}"
            );
            let expected = expected[index].as_deref().unwrap_or_else(|| {
                panic!("the fresh server published no diagnostics of document {index}")
            });
            assert_eq!(
                Self::sorted(diagnostics),
                Self::sorted(expected),
                "the diagnostics of document {index} differ from a fresh server's for {:?}",
                document.text().0
            );
        }
    }
}

impl StateMachineTest for Simulation {
    type SystemUnderTest = Self;
    type Reference = Editor;

    fn init_test(_: &Editor) -> Self::SystemUnderTest {
        Self::count_panics();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap();
        let connection = runtime.block_on(Connection::start(None));
        Self {
            runtime,
            session: Session {
                connection,
                uris: std::array::from_fn(|index| Server::uri(&format!("simulation/{index}.sql"))),
                documents: Default::default(),
                linter: None,
            },
            panics: PANICS.load(Ordering::SeqCst),
            _watchdog: Watchdog::start(),
        }
    }

    fn apply(mut state: Self, _: &Editor, action: Action) -> Self {
        // Only `Runtime::block_on` runs the tasks and the timers of a current-thread runtime.
        state.runtime.block_on(state.session.act(action));
        state
    }

    fn check_invariants(state: &Self, editor: &Editor) {
        assert_eq!(
            PANICS.load(Ordering::SeqCst),
            state.panics,
            "the server panicked, see the output"
        );
        for (document, text) in state.session.documents.iter().zip(&editor.documents) {
            assert_eq!(document.as_ref().map(ClientDocument::text), text.as_ref());
        }
    }

    fn teardown(state: Self, _: Editor) {
        let Simulation {
            runtime,
            mut session,
            ..
        } = state;
        runtime.block_on(async {
            session.finish().await;
            session.connection.stop().await;
        });
    }
}

prop_state_machine! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|cases| cases.parse().ok())
            .unwrap_or(SESSIONS_DEFAULT),
        ..ProptestConfig::default()
    })]

    #[test]
    fn simulated_sessions(sequential 1..60 => Simulation);
}
