//! A simulated editor that drives the language server through random sessions.
//!
//! A session is a list of the editor's actions: it opens, edits, replaces, saves and closes
//! documents, sends requests and cancels some of them, waits, and changes the configuration. With
//! a database, it also breaks the database and repairs it. proptest generates the sessions and
//! shrinks a failing one. The actions are interpreted leniently, so that any list is a session:
//! an action picks its document by index modulo the open ones, and is skipped if there is none.
//!
//! The server runs in process on a multi-threaded runtime, with short debounces and backoffs, on
//! the real clock. Its notifications are read after every action, in the order the server sent
//! them, and checked:
//! - every request answers within [`REQUEST_TIMEOUT`], with a result or, if it was cancelled or
//!   out of range, an error;
//! - nothing panics, counted with a panic hook, including the panics the server catches;
//! - every range and position of a response, and of the published diagnostics, is valid in the
//!   client's text of the version it is for, with lines split as LSP splits them;
//! - the diagnostics of a document never go back to an older version, and nothing is published
//!   for a document after the server published its close.
//!
//! At the end, the database is repaired and the backoffs expire, and the session waits until the
//! server is idle ([`Server::wait_until_idle`]). Then the last diagnostics of every open document
//! are for its current version and equal those of a fresh server that opens the final texts.
//!
//! The database faults use the real test database: a closed port refuses the connection, a role
//! that may not read `pg_proc` fails the snapshot with 42501, and `pg_terminate_backend` kills
//! the server's connections, which then come back.
//!
//! `PROPTEST_CASES` sets the number of sessions of each test, [`SESSIONS_DEFAULT`] and
//! [`DATABASE_SESSIONS_DEFAULT`] by default.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::collections::VecDeque;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use futures::channel::mpsc::{Receiver, channel};
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_configuration::{Merge, PartialConfiguration};
use pgls_fs::MemoryFileSystem;
use pgls_lsp::testing::{Client, Server, ServerNotification};
use pgls_lsp::{DebounceDurations, ServerFactory};
use pgls_workspace::DynRef;
use pgls_workspace::workspace::server::{BackoffDurations, WorkspaceServer};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use serde_json::{Value, from_value, json};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Connection as _, Executor, PgPool};
use tokio::task::JoinHandle;
use tower_lsp_server::jsonrpc::ErrorCode;
use tower_lsp_server::ls_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, CompletionParams, CompletionResponse,
    CompletionTextEdit, Diagnostic, DidSaveTextDocumentParams, DocumentFormattingParams,
    FormattingOptions, Hover, HoverParams, InlayHint, InlayHintParams, PartialResultParams,
    Position, PublishDiagnosticsParams, Range, SemanticTokensParams, SemanticTokensRangeParams,
    SemanticTokensRangeResult, SemanticTokensResult, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentPositionParams, TextEdit, Uri, WorkDoneProgressParams,
};

/// The number of sessions without a database, without `PROPTEST_CASES`.
const SESSIONS_DEFAULT: u32 = 64;

/// The number of sessions with a database, without `PROPTEST_CASES`. Each loads the snapshot a
/// few times, so they take longer.
const DATABASE_SESSIONS_DEFAULT: u32 = 16;

/// The longest session.
const ACTIONS_MAX: usize = 60;

/// The number of documents a session uses.
const DOCUMENTS: usize = 3;

/// How long a request may take on the real clock. A request that waits for something that never
/// happens takes all of it.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// How long the end of a session waits until the server is idle, and then for the diagnostics it
/// published.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// The debounces of the server, short so that they fire during a session.
const DEBOUNCE: Duration = Duration::from_millis(5);

/// The backoffs of the workspace, short so that a session sees them start and expire.
const BACKOFF: BackoffDurations = BackoffDurations {
    initial: Duration::from_millis(20),
    max: Duration::from_millis(80),
};

/// The longest wait of a session.
const WAIT_MILLIS_MAX: u64 = 50;

/// The notifications a session reads after each action at most. The server publishes the
/// diagnostics of every document at most a few times per action, so this is never reached;
/// a full channel would block the server until the session reads it.
const NOTIFICATIONS_BUFFERED_MAX: usize = 1024;

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
    "select relname from pg_class where relkind = 'r';\n",
];

/// What edits insert: pieces of statements, multi-byte characters, line breaks.
const SNIPPETS: &[&str] = &[
    "", " ", ";", "\n", "\r\n", "\r", "(", ")", "'", "$$", "é", "😀", "select", " from t", "1",
    ",", "--", "/*", "*/", "table",
];

/// The panics of this test binary so far, see [`Simulation::count_panics`].
static PANICS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn simulated_sessions() {
    Simulation::run(None, SESSIONS_DEFAULT);
}

#[pgls_test_utils::db_test]
async fn simulated_sessions_with_database(test_db: PgPool) {
    // Lets the snapshot fail for `test_login`, see `DatabaseFault::PermissionDenied`. Superusers,
    // like the user of the other faults, still read `pg_proc`.
    test_db
        .execute("revoke select on pg_catalog.pg_proc from public")
        .await
        .unwrap();
    let database = TestDatabase::new(&test_db);
    // A session runs on a runtime of its own.
    tokio::task::spawn_blocking(move || {
        Simulation::run(Some(database), DATABASE_SESSIONS_DEFAULT);
    })
    .await
    .unwrap();
}

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

/// What the session does to the database.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DatabaseFault {
    /// The test database, as the configuration file says.
    #[default]
    None,
    /// A port nobody listens on, which refuses the connection.
    Refused,
    /// The test database as `test_login`, which may not read `pg_proc`, so the snapshot fails
    /// with 42501 on a working connection.
    PermissionDenied,
}

/// What the editor does. `document` picks one of the open documents, modulo their number, and
/// `start`, `end` and `at` pick characters modulo the length of its text.
#[derive(Clone, Debug)]
enum Action {
    /// Opens the document `slot` modulo [`DOCUMENTS`], unless it is open.
    Open {
        slot: usize,
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
    /// Saves a document, which checks it against the database.
    Save {
        document: usize,
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
    /// Points the server at a database with the fault, through the client's overrides.
    SetDatabase(DatabaseFault),
    /// Kills the connections to the test database, which the server opens again.
    TerminateBackends,
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

    /// The actions of a session, with the database faults if `database`.
    fn strategy(database: bool) -> BoxedStrategy<Action> {
        let index = 0..DOCUMENTS * 4;
        let editor = prop_oneof![
            3 => (index.clone(), Self::text())
                .prop_map(|(slot, text)| Action::Open { slot, text }),
            8 => (index.clone(), any::<usize>(), any::<usize>(), Self::snippet()).prop_map(
                |(document, start, end, text)| Action::Edit {
                    document,
                    start,
                    end,
                    text,
                }
            ),
            1 => (index.clone(), Self::text())
                .prop_map(|(document, text)| Action::Replace { document, text }),
            1 => (index.clone(), 0..4u32, 0..200u32).prop_map(|(document, line, character)| {
                Action::EditOutOfRange {
                    document,
                    line,
                    character,
                }
            }),
            1 => index.clone().prop_map(|document| Action::Save { document }),
            1 => index.clone().prop_map(|document| Action::Close { document }),
            6 => (
                index,
                prop::sample::select(&RequestKind::ALL[..]),
                any::<usize>(),
                prop::bool::weighted(0.1),
                prop::bool::weighted(0.2),
            )
                .prop_map(|(document, request, at, out_of_range, cancel)| {
                    Action::Request {
                        document,
                        request,
                        at,
                        out_of_range,
                        cancel,
                    }
                }),
            2 => (0..WAIT_MILLIS_MAX).prop_map(|millis| Action::Wait { millis }),
            1 => any::<bool>().prop_map(|enabled| Action::SetLinter { enabled }),
            1 => Just(Action::ReloadConfiguration),
        ];
        if !database {
            return editor.boxed();
        }
        prop_oneof![
            20 => editor,
            2 => prop::sample::select(&[
                DatabaseFault::None,
                DatabaseFault::Refused,
                DatabaseFault::PermissionDenied,
            ][..])
            .prop_map(Action::SetDatabase),
            1 => Just(Action::TerminateBackends),
        ]
        .boxed()
    }
}

/// The test database of [`simulated_sessions_with_database`].
#[derive(Clone)]
struct TestDatabase {
    /// The connection options of the test's pool.
    options: PgConnectOptions,
    /// The URL of the test database.
    url: String,
    /// A port nobody listens on.
    closed_port: u16,
}

impl TestDatabase {
    fn new(pool: &PgPool) -> Self {
        let options = pool.connect_options().as_ref().clone();
        let database_url = pgls_test_utils::db::database_url().unwrap();
        let server = database_url
            .split('?')
            .next()
            .and_then(|url| url.rsplit_once('/'))
            .map(|(server, _)| server)
            .unwrap();
        let url = format!("{server}/{}", options.get_database().unwrap());
        // The port is free once the listener is dropped.
        let closed_port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        Self {
            options,
            url,
            closed_port,
        }
    }

    /// The database section of the configuration file.
    fn configuration(&self) -> PartialDatabaseConfiguration {
        PartialDatabaseConfiguration {
            connection_string: Some(self.url.clone()),
            ..Default::default()
        }
    }

    /// The database section of the overrides for `fault`. A refused connection is retried until
    /// the connection timeout, so it is as short as it gets. The test database keeps the default:
    /// the timeout bounds the wait for a connection of the pool, and the snapshot of a server
    /// that has none yet connects and queues its queries for more than a second on a busy
    /// machine. The fresh server of [`Simulation::finish`] would then check without the database.
    fn overrides(&self, fault: DatabaseFault) -> Option<Value> {
        match fault {
            DatabaseFault::None => None,
            DatabaseFault::Refused => Some(json!({
                "port": self.closed_port,
                "connTimeoutSecs": 1,
            })),
            DatabaseFault::PermissionDenied => Some(json!({
                "connectionString": format!("{}?options[role]=test_login", self.url),
            })),
        }
    }

    /// Kills every other connection to the test database.
    async fn terminate_backends(&self) {
        let mut connection = PgConnection::connect_with(&self.options).await.unwrap();
        connection
            .execute(
                "select pg_terminate_backend(pid) from pg_stat_activity \
                 where datname = current_database() and pid <> pg_backend_pid()",
            )
            .await
            .unwrap();
        connection.close().await.unwrap();
    }
}

/// The overrides the client sets: the linter and the database fault.
#[derive(Clone, Copy, Debug, Default)]
struct Overrides {
    linter: Option<bool>,
    database: DatabaseFault,
}

impl Overrides {
    /// The parameters of `pgls/set_configuration_overrides`.
    fn params(self, database: Option<&TestDatabase>) -> Value {
        let mut overrides = serde_json::Map::new();
        if let Some(enabled) = self.linter {
            overrides.insert("linter".to_string(), json!({ "enabled": enabled }));
        }
        if let Some(db) = database.and_then(|database| database.overrides(self.database)) {
            overrides.insert("db".to_string(), db);
        }
        json!({ "overrides": overrides })
    }
}

/// An open document, as the client sees it.
struct ClientDocument {
    /// The text of every version since the document was opened, by version. `None` if the
    /// client doesn't know it: after an edit out of range.
    texts: Vec<Option<ClientText>>,
    /// The newest version the server published diagnostics for, and the diagnostics.
    published: Option<(i32, Vec<Diagnostic>)>,
}

impl ClientDocument {
    fn new(text: String) -> Self {
        Self {
            texts: vec![Some(ClientText(text))],
            published: None,
        }
    }

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

    /// Checks diagnostics the server published for `version` of document `index`.
    fn check_published(&mut self, index: usize, version: i32, diagnostics: Vec<Diagnostic>) {
        if let Some((published, _)) = &self.published {
            assert!(
                *published <= version,
                "the diagnostics of version {version} of document {index} came after those of version {published}"
            );
        }
        let text = usize::try_from(version)
            .ok()
            .and_then(|version| self.texts.get(version))
            .unwrap_or_else(|| {
                panic!("diagnostics of version {version} of document {index}, which it never had")
            });
        if let Some(text) = text {
            for diagnostic in &diagnostics {
                text.check(
                    diagnostic.range,
                    &format!("diagnostic {:?}", diagnostic.message),
                );
            }
        }
        self.published = Some((version, diagnostics));
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
}

/// A document of the session, which the client may have opened and closed several times.
///
/// The server's notifications are read after the client acted, so they can be about an earlier
/// time the document was open. The server publishes empty diagnostics without a version when it
/// closes a document, which ends that time.
#[derive(Default)]
struct DocumentSlot {
    /// The times the document was open that the server may still publish diagnostics for, oldest
    /// first: those the client closed and whose close it hasn't read yet, then the open one.
    opened: VecDeque<ClientDocument>,
    /// Whether the last of `opened` is open.
    is_open: bool,
}

impl DocumentSlot {
    fn open(&self) -> Option<&ClientDocument> {
        self.opened.back().filter(|_| self.is_open)
    }

    fn open_mut(&mut self) -> Option<&mut ClientDocument> {
        self.opened.back_mut().filter(|_| self.is_open)
    }

    /// Checks diagnostics the server published for document `index`.
    fn check_published(&mut self, index: usize, params: PublishDiagnosticsParams) {
        match params.version {
            None => {
                assert!(
                    params.diagnostics.is_empty(),
                    "diagnostics without a version: {params:?}"
                );
                assert!(
                    self.opened.len() > usize::from(self.is_open),
                    "the server closed document {index}, which is open or closed already"
                );
                self.opened.pop_front();
            }
            Some(version) => {
                let Some(document) = self.opened.front_mut() else {
                    panic!("diagnostics of the closed document {index}: {params:?}");
                };
                document.check_published(index, version, params.diagnostics);
            }
        }
    }
}

/// A language server and the client that talks to it.
struct Connection {
    server: Server,
    notifications: Receiver<ServerNotification>,
    client: JoinHandle<Result<()>>,
    /// The diagnostics read from `notifications`.
    diagnostics_received: u64,
}

impl Connection {
    /// Starts a server on the current runtime with short debounces and backoffs, connected to
    /// `database` or with the database connection disabled, and sets `overrides`.
    async fn start(database: Option<&TestDatabase>, overrides: Overrides) -> Self {
        let factory = ServerFactory::default()
            .with_workspace(std::sync::Arc::new(WorkspaceServer::with_backoff(BACKOFF)))
            .with_debounce(DebounceDurations {
                diagnostics: DEBOUNCE,
                database_checks: DEBOUNCE,
                configuration_reload: DEBOUNCE,
            });
        let mut configuration = PartialConfiguration::init();
        configuration.merge_with(PartialConfiguration {
            db: Some(match database {
                Some(database) => database.configuration(),
                None => PartialDatabaseConfiguration {
                    disable_connection: Some(true),
                    ..Default::default()
                },
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
        let (sender, notifications) = channel(NOTIFICATIONS_BUFFERED_MAX);
        let client = tokio::spawn(Client::run(stream, sink, sender));
        let mut server = Server::with_timeout(service, REQUEST_TIMEOUT);
        server.initialize().await.unwrap();
        server.initialized().await.unwrap();
        server.load_configuration().await.unwrap();
        let mut connection = Self {
            server,
            notifications,
            client,
            diagnostics_received: 0,
        };
        connection.set_overrides(overrides, database).await;
        connection
    }

    async fn set_overrides(&mut self, overrides: Overrides, database: Option<&TestDatabase>) {
        self.server
            .set_configuration_overrides(overrides.params(database))
            .await
            .unwrap();
    }

    /// The diagnostics the server published that arrived.
    fn try_next_diagnostics(&mut self) -> Option<PublishDiagnosticsParams> {
        while let Ok(Some(notification)) = self.notifications.try_next() {
            if let ServerNotification::PublishDiagnostics(params) = notification {
                self.diagnostics_received += 1;
                return Some(params);
            }
        }
        None
    }

    /// Waits until the server is idle, and returns the diagnostics it published that haven't
    /// been read yet.
    async fn idle_diagnostics(&mut self) -> Vec<PublishDiagnosticsParams> {
        let progress = self.server.wait_until_idle(IDLE_TIMEOUT).await.unwrap();
        let mut published = Vec::new();
        while self.diagnostics_received < progress.diagnostics_published {
            let notification = tokio::time::timeout(IDLE_TIMEOUT, self.notifications.next())
                .await
                .expect("the published diagnostics arrive")
                .expect("the server keeps the connection");
            if let ServerNotification::PublishDiagnostics(params) = notification {
                self.diagnostics_received += 1;
                published.push(params);
            }
        }
        published
    }

    async fn stop(mut self) {
        self.server.shutdown().await.unwrap();
        self.client.abort();
    }
}

/// A session: the server and the client's documents.
struct Simulation {
    connection: Connection,
    database: Option<TestDatabase>,
    uris: [Uri; DOCUMENTS],
    documents: [DocumentSlot; DOCUMENTS],
    overrides: Overrides,
    /// The panics before the session started.
    panics: usize,
}

impl Simulation {
    /// Runs `cases` random sessions, with `database` if set, and panics with the shortest failing
    /// session that proptest finds.
    fn run(database: Option<TestDatabase>, cases: u32) {
        Self::count_panics();
        let cases = std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|cases| cases.parse().ok())
            .unwrap_or(cases);
        let mut runner = TestRunner::new(Config {
            cases,
            ..Config::default()
        });
        let sessions = prop::collection::vec(Action::strategy(database.is_some()), 1..ACTIONS_MAX);
        let result = runner.run(&sessions, |actions| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(pgls_workspace::WORKER_STACK_SIZE_BYTES)
                .build()
                .unwrap();
            runtime.block_on(async {
                let mut simulation = Self::start(database.clone()).await;
                for action in actions {
                    simulation.act(action).await;
                    simulation.check_panics();
                }
                simulation.finish().await;
            });
            Ok(())
        });
        if let Err(failure) = result {
            panic!("{failure}");
        }
    }

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

    fn check_panics(&self) {
        assert_eq!(
            PANICS.load(Ordering::SeqCst),
            self.panics,
            "the server panicked, see the output"
        );
    }

    async fn start(database: Option<TestDatabase>) -> Self {
        let connection = Connection::start(database.as_ref(), Overrides::default()).await;
        Self {
            connection,
            database,
            uris: std::array::from_fn(|index| Server::uri(&format!("simulation/{index}.sql"))),
            documents: Default::default(),
            overrides: Overrides::default(),
            panics: PANICS.load(Ordering::SeqCst),
        }
    }

    /// The document that `document` picks among the open ones.
    fn pick(&self, document: usize) -> Option<usize> {
        let open: Vec<usize> = (0..DOCUMENTS)
            .filter(|&index| self.documents[index].is_open)
            .collect();
        (!open.is_empty()).then(|| open[document % open.len()])
    }

    fn text(&self, index: usize) -> ClientText {
        self.documents[index].open().unwrap().text().clone()
    }

    async fn change(&mut self, index: usize, version: i32, change: TextDocumentContentChangeEvent) {
        self.connection
            .server
            .change_named_document(self.uris[index].clone(), version, vec![change])
            .await
            .unwrap();
    }

    async fn act(&mut self, action: Action) {
        match action {
            Action::Open { slot, text } => {
                let index = slot % DOCUMENTS;
                if self.documents[index].is_open {
                    return;
                }
                self.connection
                    .server
                    .open_named_document(&text, self.uris[index].clone(), "sql")
                    .await
                    .unwrap();
                let slot = &mut self.documents[index];
                slot.opened.push_back(ClientDocument::new(text));
                slot.is_open = true;
            }
            Action::Edit {
                document,
                start,
                end,
                text,
            } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                let current = self.text(index);
                let chars = current.char_count() + 1;
                let (start, end) = (start % chars, end % chars);
                let (range, edited) = current.edit(start.min(end), start.max(end), &text);
                let version = self.documents[index].open_mut().unwrap().push(Some(edited));
                let change = TextDocumentContentChangeEvent {
                    range: Some(range),
                    range_length: None,
                    text,
                };
                self.change(index, version, change).await;
            }
            Action::Replace { document, text } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                let version = self.documents[index]
                    .open_mut()
                    .unwrap()
                    .push(Some(ClientText(text.clone())));
                let change = TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text,
                };
                self.change(index, version, change).await;
            }
            Action::EditOutOfRange {
                document,
                line,
                character,
            } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                let text = self.text(index);
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
                let version = self.documents[index].open_mut().unwrap().push(None);
                self.change(index, version, out_of_range).await;
                let version = self.documents[index]
                    .open_mut()
                    .unwrap()
                    .push(Some(text.clone()));
                let resync = TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: text.0,
                };
                self.change(index, version, resync).await;
            }
            Action::Save { document } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                self.connection
                    .server
                    .notify(
                        "textDocument/didSave",
                        DidSaveTextDocumentParams {
                            text_document: TextDocumentIdentifier {
                                uri: self.uris[index].clone(),
                            },
                            text: None,
                        },
                    )
                    .await
                    .unwrap();
            }
            Action::Close { document } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                self.connection
                    .server
                    .close_named_document(self.uris[index].clone())
                    .await
                    .unwrap();
                self.documents[index].is_open = false;
            }
            Action::Request {
                document,
                request,
                at,
                out_of_range,
                cancel,
            } => {
                let Some(index) = self.pick(document) else {
                    return;
                };
                let uri = self.uris[index].clone();
                let text = self.text(index);
                let end = text.position(text.0.len());
                let position = if out_of_range {
                    Position::new(end.line + 2, 3)
                } else {
                    text.position(text.byte_offset(at % (text.char_count() + 1)))
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
                self.overrides.linter = Some(enabled);
                self.connection
                    .set_overrides(self.overrides, self.database.as_ref())
                    .await;
            }
            Action::ReloadConfiguration => {
                self.connection
                    .server
                    .change_watched_configuration_file()
                    .await
                    .unwrap();
            }
            Action::SetDatabase(fault) => {
                self.overrides.database = fault;
                self.connection
                    .set_overrides(self.overrides, self.database.as_ref())
                    .await;
            }
            Action::TerminateBackends => {
                if let Some(database) = &self.database {
                    database.terminate_backends().await;
                }
            }
        }
        while let Some(params) = self.connection.try_next_diagnostics() {
            self.check_published(params);
        }
    }

    fn check_published(&mut self, params: PublishDiagnosticsParams) {
        let index = self
            .uris
            .iter()
            .position(|uri| *uri == params.uri)
            .unwrap_or_else(|| panic!("diagnostics of an unknown document: {:?}", params.uri));
        self.documents[index].check_published(index, params);
    }

    /// Repairs the database, waits until the server is idle, and checks that the last diagnostics
    /// of every open document are for its current version, and equal those of a fresh server that
    /// opens the final texts.
    async fn finish(mut self) {
        if self.database.is_some() {
            self.overrides.database = DatabaseFault::None;
            self.connection
                .set_overrides(self.overrides, self.database.as_ref())
                .await;
            // The backoffs of the failures so far expire, and the reload checks every document
            // against the database again.
            tokio::time::sleep(BACKOFF.max * 2).await;
            self.connection
                .server
                .change_watched_configuration_file()
                .await
                .unwrap();
        }
        for params in self.connection.idle_diagnostics().await {
            self.check_published(params);
        }

        let mut fresh = Connection::start(self.database.as_ref(), self.overrides).await;
        for (index, slot) in self.documents.iter().enumerate() {
            if let Some(document) = slot.open() {
                fresh
                    .server
                    .open_named_document(&document.text().0, self.uris[index].clone(), "sql")
                    .await
                    .unwrap();
            }
        }
        let mut expected: [Option<Vec<Diagnostic>>; DOCUMENTS] = Default::default();
        for params in fresh.idle_diagnostics().await {
            if let Some(index) = self.uris.iter().position(|uri| *uri == params.uri) {
                expected[index] = Some(params.diagnostics);
            }
        }
        fresh.stop().await;

        for (index, slot) in self.documents.iter().enumerate() {
            assert_eq!(
                slot.opened.len(),
                usize::from(slot.is_open),
                "the server didn't publish every close of document {index}"
            );
            let Some(document) = slot.open() else {
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
                ClientDocument::sorted(diagnostics),
                ClientDocument::sorted(expected),
                "the diagnostics of document {index} differ from a fresh server's for {:?}",
                document.text().0
            );
        }
        self.check_panics();
        self.connection.stop().await;
    }
}
