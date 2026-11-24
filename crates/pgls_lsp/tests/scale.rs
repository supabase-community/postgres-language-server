//! The language server on generated files of 1k and 20k statements (S8): the line indexes a
//! keystroke builds, and every request on the 20k-statement file on threads with the stack size
//! of the released server, which finds recursion whose depth grows with the file.
//!
//! The 20k-statement test is ignored, and the `Scale` job of CI runs it in release.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::channel;
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_configuration::{Merge, PartialConfiguration, PartialFormatConfiguration};
use pgls_fs::MemoryFileSystem;
use pgls_lsp::ServerFactory;
use pgls_lsp::testing::{Client, Server};
use pgls_test_utils::scale::Dump;
use pgls_workspace::DynRef;
use serde_json::Value;
use tower_lsp_server::ls_types::{
    CompletionParams, DocumentFormattingParams, FormattingOptions, HoverParams, Position, Range,
    SemanticTokensParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentPositionParams,
};

/// How long the server may take for the 20k-statement file, e.g. the first pull.
const SCALE_TIMEOUT: Duration = Duration::from_secs(600);

/// A server without a database and with the formatter on, with `dump` open as `document.sql`.
struct Connection {
    server: Server,
    dump: Dump,
}

impl Connection {
    async fn open(dump: Dump) -> Self {
        let mut configuration = PartialConfiguration::init();
        configuration.merge_with(PartialConfiguration {
            db: Some(PartialDatabaseConfiguration {
                disable_connection: Some(true),
                ..Default::default()
            }),
            format: Some(PartialFormatConfiguration {
                enabled: Some(true),
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
        let (service, socket) = ServerFactory::default()
            .create_with_fs(None, DynRef::Owned(Box::new(fs)))
            .into_inner();
        let (stream, sink) = socket.split();
        let (sender, mut notifications) = channel(Client::CHANNEL_BUFFER_SIZE);
        tokio::spawn(Client::run(stream, sink, sender));
        // Read the published diagnostics, so that the server never waits for the client.
        tokio::spawn(async move { while notifications.next().await.is_some() {} });

        let mut server = Server::with_timeout(service, SCALE_TIMEOUT);
        server.initialize().await.unwrap();
        server.initialized().await.unwrap();
        server.load_configuration().await.unwrap();
        server.open_document(&dump.sql).await.unwrap();
        server.wait_until_idle(SCALE_TIMEOUT).await.unwrap();
        Self { server, dump }
    }

    /// The position of `offset` in the dump, which is ASCII.
    fn position(&self, offset: usize) -> Position {
        let before = &self.dump.sql[..offset];
        let line = before.matches('\n').count();
        let character = offset - before.rfind('\n').map_or(0, |newline| newline + 1);
        Position::new(line.try_into().unwrap(), character.try_into().unwrap())
    }

    fn document() -> TextDocumentIdentifier {
        TextDocumentIdentifier {
            uri: Server::uri("document.sql"),
        }
    }

    fn at(&self, offset: usize) -> TextDocumentPositionParams {
        TextDocumentPositionParams {
            text_document: Self::document(),
            position: self.position(offset),
        }
    }

    /// Types a space at `offset`, and returns how many line indexes the server built for it.
    async fn type_space(&mut self, version: i32, offset: usize) -> u64 {
        let before = self.server.wait_until_idle(SCALE_TIMEOUT).await.unwrap();
        let position = self.position(offset);
        self.server
            .change_document(
                version,
                vec![TextDocumentContentChangeEvent {
                    range: Some(Range::new(position, position)),
                    range_length: None,
                    text: " ".to_owned(),
                }],
            )
            .await
            .unwrap();
        let after = self.server.wait_until_idle(SCALE_TIMEOUT).await.unwrap();
        after.line_indexes_built - before.line_indexes_built
    }
}

/// Row 11 of the S8 budgets: a keystroke builds a line index to apply the change and one of the
/// new text, not one per range of the change or per request.
#[tokio::test]
async fn a_keystroke_builds_at_most_two_line_indexes() {
    let dump = Dump::new(1_000);
    let offsets = dump.edit_offsets();
    let mut connection = Connection::open(dump).await;
    for (version, offset) in (1..).zip(offsets) {
        let built = connection.type_space(version, offset).await;
        assert!(built <= 2, "a keystroke built {built} line indexes");
    }
}

/// Every request on a 20k-statement file, on threads with the stack of the released server.
#[test]
#[ignore = "slow in debug; CI's Scale job runs it in release"]
fn scale_20k_statements_on_the_production_stack() {
    let stack = pgls_workspace::WORKER_STACK_SIZE_BYTES;
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(stack)
                .build()
                .unwrap()
                .block_on(Connection::scale_20k())
        })
        .unwrap()
        .join()
        .expect("the server ran out of stack or panicked");
}

impl Connection {
    async fn scale_20k() {
        let dump = Dump::new(20_000);
        let last_query = dump.query(1.0).unwrap();
        let hover_at = last_query + dump.sql[last_query..].find("from public.t").unwrap() + 12;
        let offsets = dump.edit_offsets();
        let mut connection = Connection::open(dump).await;

        let hover = connection.at(hover_at);
        connection
            .server
            .request::<_, Value>(
                "textDocument/hover",
                "hover",
                HoverParams {
                    text_document_position_params: hover,
                    work_done_progress_params: Default::default(),
                },
            )
            .await
            .unwrap();
        let completion = connection.at(last_query + "select t.".len());
        connection
            .server
            .get_completion(CompletionParams {
                text_document_position: completion,
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: None,
            })
            .await
            .unwrap();
        connection
            .server
            .request::<_, Value>(
                "textDocument/semanticTokens/full",
                "semantic_tokens",
                SemanticTokensParams {
                    text_document: Self::document(),
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                },
            )
            .await
            .unwrap();
        connection
            .server
            .request::<_, Value>(
                "textDocument/formatting",
                "formatting",
                DocumentFormattingParams {
                    text_document: Self::document(),
                    options: FormattingOptions {
                        tab_size: 4,
                        insert_spaces: true,
                        ..Default::default()
                    },
                    work_done_progress_params: Default::default(),
                },
            )
            .await
            .unwrap();
        for (version, offset) in (1..).zip(offsets) {
            connection.type_space(version, offset).await;
        }
        connection.server.shutdown().await.unwrap();
    }
}
