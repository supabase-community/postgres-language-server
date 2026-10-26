use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use anyhow::bail;
use futures::Sink;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};
use pgls_configuration::Merge;
use pgls_configuration::PartialConfiguration;
use pgls_configuration::StringSet;
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_configuration::migrations::{PartialMigrationsConfiguration, SectionMarkers};
use pgls_fs::{ConfigName, MemoryFileSystem, OsFileSystem};
use pgls_lsp::LSPServer;
use pgls_lsp::ServerFactory;
use pgls_workspace::DynRef;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::{from_value, json, to_value};
use sqlx::Executor;
use sqlx::PgPool;
use std::any::type_name;
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tower::timeout::Timeout;
use tower::{Service, ServiceExt};
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc;
use tower_lsp_server::jsonrpc::Response;
use tower_lsp_server::ls_types as lsp;
use tower_lsp_server::ls_types::CodeActionContext;
use tower_lsp_server::ls_types::CodeActionParams;
use tower_lsp_server::ls_types::CodeActionResponse;
use tower_lsp_server::ls_types::CompletionParams;
use tower_lsp_server::ls_types::CompletionResponse;
use tower_lsp_server::ls_types::ExecuteCommandParams;
use tower_lsp_server::ls_types::PartialResultParams;
use tower_lsp_server::ls_types::Position;
use tower_lsp_server::ls_types::Range;
use tower_lsp_server::ls_types::TextDocumentPositionParams;
use tower_lsp_server::ls_types::WorkDoneProgressParams;
use tower_lsp_server::ls_types::WorkspaceFolder;
use tower_lsp_server::ls_types::{
    ClientCapabilities, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, FileChangeType, FileEvent, InitializeResult, InitializedParams,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, Uri, VersionedTextDocumentIdentifier, WorkspaceFoldersChangeEvent,
};
use tower_lsp_server::{jsonrpc::Request, ls_types::InitializeParams};

/// Statically build an [Uri] instance that points to the file at `$path`
/// within the workspace. The filesystem path contained in the return URI is
/// guaranteed to be a valid path for the underlying operating system, but
/// doesn't have to refer to an existing file on the host machine.
macro_rules! url {
    ($path:literal) => {
        if cfg!(windows) {
            lsp::Uri::from_str(concat!("file:///z%3A/workspace/", $path)).unwrap()
        } else {
            lsp::Uri::from_str(concat!("file:///workspace/", $path)).unwrap()
        }
    };
}

struct Server {
    service: Timeout<LspService<LSPServer>>,
}

impl Server {
    fn new(service: LspService<LSPServer>) -> Self {
        Self {
            service: Timeout::new(service, Duration::from_secs(1)),
        }
    }

    async fn notify<P>(&mut self, method: &'static str, params: P) -> Result<()>
    where
        P: Serialize,
    {
        self.service
            .ready()
            .await
            .map_err(Error::msg)
            .context("ready() returned an error")?
            .call(
                Request::build(method)
                    .params(to_value(&params).context("failed to serialize params")?)
                    .finish(),
            )
            .await
            .map_err(Error::msg)
            .context("call() returned an error")
            .and_then(|res| match res {
                Some(res) => {
                    bail!("shutdown returned {res:?}")
                }
                _ => Ok(()),
            })
    }

    async fn request<P, R>(
        &mut self,
        method: &'static str,
        id: &'static str,
        params: P,
    ) -> Result<Option<R>>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.service
            .ready()
            .await
            .map_err(Error::msg)
            .context("ready() returned an error")?
            .call(
                Request::build(method)
                    .id(id)
                    .params(to_value(&params).context("failed to serialize params")?)
                    .finish(),
            )
            .await
            .map_err(Error::msg)
            .context("call() returned an error")?
            .map(|res| {
                let (_, body) = res.into_parts();

                let body =
                    body.with_context(|| format!("response to {method:?} contained an error"))?;

                from_value(body.clone()).with_context(|| {
                    format!(
                        "failed to deserialize type {} from response {body:?}",
                        type_name::<R>()
                    )
                })
            })
            .transpose()
    }

    /// Basic implementation of the `initialize` request for tests
    async fn initialize(&mut self) -> Result<()> {
        self.initialize_with_root(url!("")).await
    }

    /// Like [Server::initialize], with the given client capabilities
    async fn initialize_with_capabilities(
        &mut self,
        capabilities: ClientCapabilities,
    ) -> Result<()> {
        self.initialize_with_options(url!(""), None, capabilities)
            .await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri`
    async fn initialize_with_root(&mut self, root_uri: Uri) -> Result<()> {
        self.initialize_with_folders(root_uri, None).await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri` and the given workspace
    /// folders
    async fn initialize_with_folders(
        &mut self,
        root_uri: Uri,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
    ) -> Result<()> {
        self.initialize_with_options(root_uri, workspace_folders, ClientCapabilities::default())
            .await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri`, the given workspace
    /// folders and the given client capabilities
    // The `root_path` field is deprecated, but we still need to specify it
    #[allow(deprecated)]
    async fn initialize_with_options(
        &mut self,
        root_uri: Uri,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
        capabilities: ClientCapabilities,
    ) -> Result<()> {
        let _res: InitializeResult = self
            .request(
                "initialize",
                "_init",
                InitializeParams {
                    process_id: None,
                    root_path: None,
                    root_uri: Some(root_uri),
                    initialization_options: None,
                    capabilities,
                    trace: None,
                    workspace_folders,
                    client_info: None,
                    locale: None,
                    work_done_progress_params: Default::default(),
                },
            )
            .await?
            .context("initialize returned None")?;

        Ok(())
    }

    /// It creates two workspaces, one at folder `test_one` and the other in `test_two`.
    ///
    /// Hence, the two roots will be `/workspace/test_one` and `/workspace/test_two`
    #[allow(deprecated)]
    async fn initialize_workspaces(&mut self) -> Result<()> {
        let _res: InitializeResult = self
            .request(
                "initialize",
                "_init",
                InitializeParams {
                    process_id: None,
                    root_path: None,
                    root_uri: Some(url!("/")),
                    initialization_options: None,
                    capabilities: ClientCapabilities::default(),
                    trace: None,
                    workspace_folders: Some(vec![
                        WorkspaceFolder {
                            name: "test_one".to_string(),
                            uri: url!("test_one"),
                        },
                        WorkspaceFolder {
                            name: "test_two".to_string(),
                            uri: url!("test_two"),
                        },
                    ]),
                    client_info: None,
                    locale: None,
                    work_done_progress_params: Default::default(),
                },
            )
            .await?
            .context("initialize returned None")?;

        Ok(())
    }

    /// Basic implementation of the `initialized` notification for tests
    async fn initialized(&mut self) -> Result<()> {
        self.notify("initialized", InitializedParams {}).await
    }

    /// Basic implementation of the `shutdown` notification for tests
    async fn shutdown(&mut self) -> Result<()> {
        self.service
            .ready()
            .await
            .map_err(Error::msg)
            .context("ready() returned an error")?
            .call(Request::build("shutdown").finish())
            .await
            .map_err(Error::msg)
            .context("call() returned an error")
            .and_then(|res| match res {
                Some(res) => {
                    bail!("shutdown returned {res:?}")
                }
                _ => Ok(()),
            })
    }

    async fn open_document(&mut self, text: impl Display) -> Result<()> {
        self.notify(
            "textDocument/didOpen",
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: url!("document.sql"),
                    language_id: String::from("sql"),
                    version: 0,
                    text: text.to_string(),
                },
            },
        )
        .await
    }

    /// Opens a document with given contents and given name. The name must contain the extension too
    async fn open_named_document(
        &mut self,
        text: impl Display,
        document_name: Uri,
        language: impl Display,
    ) -> Result<()> {
        self.notify(
            "textDocument/didOpen",
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: document_name,
                    language_id: language.to_string(),
                    version: 0,
                    text: text.to_string(),
                },
            },
        )
        .await
    }

    /// When calling this function, remember to insert the file inside the memory file system
    async fn load_configuration(&mut self) -> Result<()> {
        self.notify(
            "workspace/didChangeConfiguration",
            DidChangeConfigurationParams {
                settings: to_value(()).unwrap(),
            },
        )
        .await
    }

    async fn change_named_document(
        &mut self,
        uri: Uri,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        self.notify(
            "textDocument/didChange",
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier { uri, version },
                content_changes,
            },
        )
        .await
    }

    async fn change_document(
        &mut self,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        self.change_named_document(url!("document.sql"), version, content_changes)
            .await
    }

    #[allow(unused)]
    async fn close_document(&mut self) -> Result<()> {
        self.notify(
            "textDocument/didClose",
            DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
            },
        )
        .await
    }

    async fn get_completion(
        &mut self,
        params: tower_lsp_server::ls_types::CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        self.request::<tower_lsp_server::ls_types::CompletionParams, CompletionResponse>(
            "textDocument/completion",
            "_get_completion",
            params,
        )
        .await
    }

    /// Basic implementation of the `pgls/shutdown` request for tests
    async fn pgls_shutdown(&mut self) -> Result<()> {
        self.request::<_, ()>("pgls/shutdown", "_pgls_shutdown", ())
            .await?
            .context("pgls/shutdown returned None")?;
        Ok(())
    }

    /// Sets or clears the sticky client configuration overrides
    async fn set_configuration_overrides(&mut self, params: Value) -> Result<()> {
        self.request::<_, ()>(
            "pgls/set_configuration_overrides",
            "_set_configuration_overrides",
            params,
        )
        .await?
        .context("pgls/set_configuration_overrides returned None")?;
        Ok(())
    }

    /// Tells the server that the configuration file on disk changed, which triggers a configuration
    /// reload that carries no client configuration at all
    async fn change_watched_configuration_file(&mut self) -> Result<()> {
        self.notify(
            "workspace/didChangeWatchedFiles",
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: url!("postgres-language-server.jsonc"),
                    typ: FileChangeType::CHANGED,
                }],
            },
        )
        .await
    }

    /// Replaces the whole content of `document.sql`, which schedules a fresh diagnostics run
    async fn replace_document(&mut self, version: i32, text: impl Display) -> Result<()> {
        self.change_document(
            version,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        )
        .await
    }
}

/// Number of notifications buffered by the server-to-client channel before it starts blocking the current task
const CHANNEL_BUFFER_SIZE: usize = 8;

#[derive(Debug, PartialEq, Eq)]
enum ServerNotification {
    PublishDiagnostics(PublishDiagnosticsParams),
}

/// Basic handler for requests and notifications coming from the server for tests
async fn client_handler<I, O>(
    mut stream: I,
    mut sink: O,
    mut notify: Sender<ServerNotification>,
) -> Result<()>
where
    // This function has to be generic as `RequestStream` and `ResponseSink`
    // are not exported from `tower_lsp_server` and cannot be named in the signature
    I: Stream<Item = Request> + Unpin,
    O: Sink<Response> + Unpin,
{
    while let Some(req) = stream.next().await {
        if req.method() == "textDocument/publishDiagnostics" {
            let params = req.params().expect("invalid request");
            let diagnostics = from_value(params.clone()).expect("invalid params");
            let notification = ServerNotification::PublishDiagnostics(diagnostics);
            match notify.send(notification).await {
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        let id = match req.id() {
            Some(id) => id,
            None => continue,
        };

        let res = Response::from_error(id.clone(), jsonrpc::Error::method_not_found());

        sink.send(res).await.ok();
    }

    Ok(())
}

#[tokio::test]
async fn basic_lifecycle() -> Result<()> {
    let factory = ServerFactory::default();
    let (service, client) = factory.create(None).into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

/// A client that goes away while requests are still in flight must not take the server down.
/// tower-lsp 0.20 hit an `unreachable!()` once a response was produced after writing to the
/// client had failed (ebkalderon/tower-lsp#413).
#[tokio::test]
async fn client_disconnecting_mid_request_does_not_panic() -> Result<()> {
    let factory = ServerFactory::default();
    let connection = factory.create(None);

    let (mut client_write, server_read) = tokio::io::duplex(64 * 1024);
    let (server_write, client_read) = tokio::io::duplex(64);
    // Nobody reads the responses anymore, so the first one the server writes fails.
    drop(client_read);

    let server = tokio::spawn(connection.accept(server_read, server_write));

    // The server answers these with an error since it is not initialized, which is all it takes
    // to have responses ready after the write to the client failed.
    for id in 0..32 {
        let body =
            json!({ "jsonrpc": "2.0", "id": id, "method": "textDocument/hover", "params": {} })
                .to_string();
        let message = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        client_write.write_all(message.as_bytes()).await?;
    }
    drop(client_write);

    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .context("the server did not stop after the client disconnected")?
        .context("the server panicked after the client disconnected")?;

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_database_connection(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server
        .open_document("select unknown from public.users; ")
        .await?;

    // in this test, we want to ensure a database connection is established and the snapshot is
    // loaded. This is the case when the server sends typecheck diagnostics for the query above.
    // so we wait for diagnostics to be sent.
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver.next().await {
                Some(ServerNotification::PublishDiagnostics(msg)) => {
                    if msg.diagnostics.iter().any(is_database_backed) {
                        return true;
                    }
                }
                _ => continue,
            }
        }
    })
    .await
    .is_ok();

    assert!(notification, "expected diagnostics for unknown column");

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

/// The type check error below can only be produced with a database connection and the schema of the
/// test database.
fn is_database_backed(diagnostic: &lsp::Diagnostic) -> bool {
    matches!(
        &diagnostic.code,
        Some(lsp::NumberOrString::String(code)) if code == "lint/unknownColumn"
    )
}

/// Drops every notification that is already queued, so that a later assertion cannot be satisfied
/// by a stale one.
fn drain_notifications(receiver: &mut Receiver<ServerNotification>) {
    while let Ok(Some(_)) = receiver.try_next() {}
}

/// Waits for the diagnostics of the given document version.
async fn wait_for_diagnostics(
    receiver: &mut Receiver<ServerNotification>,
    version: i32,
) -> Result<PublishDiagnosticsParams> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let Some(ServerNotification::PublishDiagnostics(params)) = receiver.next().await else {
                bail!("the server closed the connection");
            };
            if params.version == Some(version) {
                return Ok(params);
            }
        }
    })
    .await
    .with_context(|| format!("timed out waiting for diagnostics of version {version}"))?
}

/// The configuration file has neither a `db` nor a `typecheck` section and there is no environment
/// configuration, so the client override layer is the only thing that can enable database features
/// here — and clearing it has to disable them again.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn client_configuration_overrides_are_sticky_and_clearable(test_db: PgPool) -> Result<()> {
    const STATEMENT: &str = "select unknown from public.users;";

    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        "{}".to_string(),
    );

    test_db
        .execute(
            r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#,
        )
        .await
        .expect("Failed to setup test database");

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.open_document(STATEMENT).await?;

    let options = test_db.connect_options();
    server
        .set_configuration_overrides(json!({
            "overrides": {
                "db": {
                    "host": options.get_host(),
                    "port": options.get_port(),
                    "username": "postgres",
                    "password": "postgres",
                    "database": options.get_database().unwrap(),
                }
            }
        }))
        .await?;

    // The request acknowledged the overrides, and they reached the workspace: the diagnostic below
    // requires a connection to the test database.
    let connected = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver.next().await {
                Some(ServerNotification::PublishDiagnostics(params)) => {
                    if params.diagnostics.iter().any(is_database_backed) {
                        return true;
                    }
                }
                None => return false,
            }
        }
    })
    .await
    .unwrap_or(false);
    assert!(
        connected,
        "expected a database-backed diagnostic after setting the configuration overrides"
    );

    // A configuration reload carries no client configuration, but the sticky layer survives it.
    drain_notifications(&mut receiver);
    server.change_watched_configuration_file().await?;
    // The reload refreshes the diagnostics of the open document.
    wait_for_diagnostics(&mut receiver, 0).await?;
    drain_notifications(&mut receiver);
    server.replace_document(1, STATEMENT).await?;
    assert!(
        wait_for_diagnostics(&mut receiver, 1)
            .await?
            .diagnostics
            .iter()
            .any(is_database_backed),
        "the overrides should have survived a configuration reload"
    );

    // Clearing them falls back to the configuration file, which has no database at all.
    server
        .set_configuration_overrides(json!({ "overrides": null }))
        .await?;
    drain_notifications(&mut receiver);
    server.replace_document(2, STATEMENT).await?;
    assert!(
        !wait_for_diagnostics(&mut receiver, 2)
            .await?
            .diagnostics
            .iter()
            .any(is_database_backed),
        "clearing the overrides should have disabled the database features again"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[tokio::test]
async fn server_shutdown() -> Result<()> {
    let factory = ServerFactory::default();
    let (service, client) = factory.create(None).into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    let cancellation = factory.cancellation();
    let cancellation = cancellation.notified();

    // this is called when `postgrestools stop` is run by the user
    server.pgls_shutdown().await?;

    cancellation.await;

    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_completions(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server
        .open_document("alter table appointment alter column end_time drop not null;\n")
        .await?;

    server
        .change_document(
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 24,
                    },
                    end: Position {
                        line: 0,
                        character: 24,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    let res = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 25,
                },
            },
        })
        .await?;

    let list = match res.context("expected a completion response")? {
        CompletionResponse::List(l) => l,
        other => bail!("expected CompletionResponse::List, got {other:?}"),
    };

    assert!(!list.items.is_empty(), "expected completion items");
    assert!(!list.is_incomplete, "result should not be truncated");

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_inlay_hints(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    test_db
        .execute(
            "create function make_point(x int, y int) returns int language sql as 'select x + y';",
        )
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server
        .open_document("select 1;\nselect make_point(1, y => 2);\n")
        .await?;

    let hints: Option<Vec<lsp::InlayHint>> = server
        .request(
            "textDocument/inlayHint",
            "inlay_hints",
            lsp::InlayHintParams {
                work_done_progress_params: WorkDoneProgressParams::default(),
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(2, 0),
                },
            },
        )
        .await?;

    let hints = hints.context("expected inlay hints")?;
    assert_eq!(hints.len(), 1, "expected one hint, got {hints:?}");
    assert_eq!(hints[0].position, Position::new(1, 18));
    assert!(matches!(&hints[0].label, lsp::InlayHintLabel::String(label) if label == "x:"));
    assert_eq!(hints[0].kind, Some(lsp::InlayHintKind::PARAMETER));

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_completions_is_incomplete_when_truncated(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    // Create 55 tables — more than pgls_completions::LIMIT (50)
    // so table-name completions at "select * from " will exceed the limit
    // and be truncated.
    let setup = r#"
        do $$
        begin
            for i in 0..54 loop
                execute format('create table public.t%s (id int);', lpad(i::text, 3, '0'));
            end loop;
        end
        $$;
    "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;
    server.load_configuration().await?;

    server.open_document("select * from t\n").await?;

    let res = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 15,
                },
            },
        })
        .await?;

    let list = match res.context("expected a completion response")? {
        CompletionResponse::List(l) => l,
        other => bail!("expected CompletionResponse::List, got {other:?}"),
    };

    assert_eq!(
        list.items.len(),
        pgls_completions::LIMIT,
        "items should be truncated to LIMIT"
    );
    assert!(
        list.is_incomplete,
        "is_incomplete must be true when results are truncated at LIMIT"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[tokio::test]
async fn test_completions_is_complete_when_no_db_configured() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    // No DB connection configured.
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            disable_connection: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;
    server.load_configuration().await?;

    server.open_document("select * from users\n").await?;

    let res = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 14,
                },
            },
        })
        .await?;

    let list = match res.context("expected a completion response")? {
        CompletionResponse::List(l) => l,
        other => bail!("expected CompletionResponse::List, got {other:?}"),
    };

    assert!(
        list.items.is_empty(),
        "expected no items without a DB connection, got {}",
        list.items.len()
    );
    assert!(
        !list.is_incomplete,
        "is_incomplete must be false when items is empty due to no DB being configured (0 != LIMIT)"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_issue_271(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server
        .open_document("CREATE COLLATION ignore_accent_case (provider = icu, deterministic = false, locale = 'und-u-ks-level1');\n\n-- CREATE OR REPLACE FUNCTION\n--     add_one(integer)\n-- RETURNS\n--     integer\n-- AS\n--     'add_one.so', 'add_one'\n-- LANGUAGE\n--     C \n-- STRICT;\n\n\nSELECT pwhash, FROM users;")
        .await?;

    server
        .change_document(
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 13,
                    },
                    end: Position {
                        line: 13,
                        character: 14,
                    },
                }),
                range_length: Some(0),
                text: "".to_string(),
            }],
        )
        .await?;

    server
        .change_document(
            1,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 13,
                    },
                    end: Position {
                        line: 13,
                        character: 13,
                    },
                }),
                range_length: Some(0),
                text: ",".to_string(),
            }],
        )
        .await?;

    server
        .change_document(
            2,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 14,
                    },
                    end: Position {
                        line: 13,
                        character: 14,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    server
        .change_document(
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 15,
                    },
                    end: Position {
                        line: 13,
                        character: 15,
                    },
                }),
                range_length: Some(0),
                text: "county_name".to_string(),
            }],
        )
        .await?;

    server
        .change_document(
            4,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 13,
                    },
                    end: Position {
                        line: 13,
                        character: 26,
                    },
                }),
                range_length: Some(13),
                text: "".to_string(),
            }],
        )
        .await?;

    server
        .change_document(
            5,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 13,
                        character: 13,
                    },
                    end: Position {
                        line: 13,
                        character: 13,
                    },
                }),
                range_length: Some(0),
                text: ",".to_string(),
            }],
        )
        .await?;

    // crashes with range end index 37 out of range for slice of length 26
    let res = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 13,
                    character: 14,
                },
            },
        })
        .await?;

    assert!(res.is_some());

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_execute_statement(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let database = test_db
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    let host = test_db.connect_options().get_host().to_string();

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(database),
            host: Some(host),
            ..Default::default()
        }),
        ..Default::default()
    });

    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    let users_tbl_exists = async || {
        sqlx::query_scalar::<_, bool>(
            r#"
            select exists (
                select 1
                from pg_catalog.pg_tables
                where tablename = 'users'
            )
        "#,
        )
        .fetch_one(&test_db.clone())
        .await
        .unwrap()
    };

    assert!(
        !(users_tbl_exists().await),
        "The user table shouldn't exist at this point."
    );

    let doc_content = r#"
        create table users (
            id serial primary key,
            name text,
            email text
        );
    "#;

    let doc_url = url!("test.sql");

    server
        .open_named_document(doc_content.to_string(), doc_url.clone(), "sql")
        .await?;

    let code_actions_response = server
        .request::<CodeActionParams, CodeActionResponse>(
            "textDocument/codeAction",
            "_code_action",
            CodeActionParams {
                text_document: TextDocumentIdentifier {
                    uri: doc_url.clone(),
                },
                range: Range {
                    start: Position::new(3, 7),
                    end: Position::new(3, 7),
                }, // just somewhere within the statement.
                context: CodeActionContext::default(),
                partial_result_params: PartialResultParams::default(),
                work_done_progress_params: WorkDoneProgressParams::default(),
            },
        )
        .await?
        .unwrap();

    let exec_statement_command: (String, Vec<Value>) = code_actions_response
        .iter()
        .find_map(|action_or_cmd| match action_or_cmd {
            lsp::CodeActionOrCommand::CodeAction(code_action) => {
                let command = code_action.command.as_ref();
                if command.is_some_and(|cmd| &cmd.command == "pgls.executeStatement") {
                    let command = command.unwrap();
                    let arguments = command.arguments.as_ref().unwrap().clone();
                    Some((command.command.clone(), arguments))
                } else {
                    None
                }
            }

            _ => None,
        })
        .expect("Did not find executeStatement command!");

    server
        .request::<ExecuteCommandParams, Option<Value>>(
            "workspace/executeCommand",
            "_execStmt",
            ExecuteCommandParams {
                command: exec_statement_command.0,
                arguments: exec_statement_command.1,
                ..Default::default()
            },
        )
        .await?;

    assert!(
        users_tbl_exists().await,
        "Users table did not exists even though it should've been created by the workspace/executeStatement command."
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_lint_fix_code_actions(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let database = test_db
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    let host = test_db.connect_options().get_host().to_string();

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(database),
            host: Some(host),
            ..Default::default()
        }),
        ..Default::default()
    });

    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    let users_tbl_exists = async || {
        sqlx::query_scalar::<_, bool>(
            r#"
            select exists (
                select 1
                from pg_catalog.pg_tables
                where tablename = 'users'
            )
        "#,
        )
        .fetch_one(&test_db.clone())
        .await
        .unwrap()
    };

    assert!(
        !(users_tbl_exists().await),
        "The user table shouldn't exist at this point."
    );

    let doc_content = "reindex index my_index;";
    let doc_url = url!("test.sql");
    server
        .open_named_document(doc_content.to_string(), doc_url.clone(), "sql")
        .await?;

    let code_actions_response = server
        .request::<CodeActionParams, CodeActionResponse>(
            "textDocument/codeAction",
            "_code_action",
            CodeActionParams {
                text_document: TextDocumentIdentifier {
                    uri: doc_url.clone(),
                },
                range: Range {
                    start: Position::new(0, 3),
                    end: Position::new(0, 3),
                },
                context: CodeActionContext::default(),
                partial_result_params: PartialResultParams::default(),
                work_done_progress_params: WorkDoneProgressParams::default(),
            },
        )
        .await?
        .unwrap();

    let actions: Vec<lsp::CodeAction> = code_actions_response
        .into_iter()
        .filter_map(|action| match action {
            lsp::CodeActionOrCommand::CodeAction(action) => Some(action),
            lsp::CodeActionOrCommand::Command(_) => None,
        })
        .collect();

    let fix = actions
        .iter()
        .find(|action| action.title == "Add CONCURRENTLY")
        .expect("expected a fix for requireConcurrentReindex");
    assert_eq!(fix.is_preferred, Some(true));
    let edits = &fix.edit.as_ref().unwrap().changes.as_ref().unwrap()[&doc_url];
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, " CONCURRENTLY");
    assert_eq!(edits[0].range.start, Position::new(0, 13));

    let suppression = actions
        .iter()
        .find(|action| action.title == "Suppress rule requireConcurrentReindex for this statement")
        .expect("expected a suppression action");
    let edits = &suppression.edit.as_ref().unwrap().changes.as_ref().unwrap()[&doc_url];
    assert_eq!(
        edits[0].new_text,
        "-- pgls-ignore requireConcurrentReindex\n"
    );
    assert_eq!(edits[0].range.start, Position::new(0, 0));

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_invalidate_schema_cache(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let database = test_db
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    let host = test_db.connect_options().get_host().to_string();

    // Setup: Create a table with only id column (no name column yet)
    let setup = r#"
        create table public.users (
            id serial primary key
        );
    "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(database),
            host: Some(host),
            ..Default::default()
        }),
        ..Default::default()
    });

    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    // Open a document to get completions from
    let doc_content = "select  from public.users;";
    server.open_document(doc_content).await?;

    // Get completions before adding the column - 'name' should NOT be present
    let completions_before = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 7,
                },
            },
        })
        .await?
        .unwrap();

    let items_before = match completions_before {
        CompletionResponse::Array(ref a) => a,
        CompletionResponse::List(ref l) => &l.items,
    };

    let has_name_before = items_before.iter().any(|item| {
        item.label == "name"
            && item.label_details.as_ref().is_some_and(|d| {
                d.description
                    .as_ref()
                    .is_some_and(|desc| desc.contains("public.users"))
            })
    });

    assert!(
        !has_name_before,
        "Column 'name' should not be in completions before it's added to the table"
    );

    // Add the missing column to the database
    let alter_table = r#"
        alter table public.users
        add column name text;
    "#;

    test_db
        .execute(alter_table)
        .await
        .expect("Failed to add column to table");

    // Invalidate the schema cache (all = false for current connection only)
    server
        .request::<bool, ()>("pgls/invalidate_schema_cache", "_invalidate_cache", false)
        .await?;

    // Get completions after invalidating cache - 'name' should NOW be present
    let completions_after = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 7,
                },
            },
        })
        .await?
        .unwrap();

    let items_after = match completions_after {
        CompletionResponse::Array(ref a) => a,
        CompletionResponse::List(ref l) => &l.items,
    };

    let has_name_after = items_after.iter().any(|item| {
        item.label == "name"
            && item.label_details.as_ref().is_some_and(|d| {
                d.description
                    .as_ref()
                    .is_some_and(|desc| desc.contains("public.users"))
            })
    });

    assert!(
        has_name_after,
        "Column 'name' should be in completions after schema cache invalidation"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_issue_281(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server.open_document("\n------------- Meta -------------\n\n-- name: GetValueFromMetaKVStore :one\nSELECT value FROM meta_kv\nWHERE key = $1;\n\n-- name: SetValueToMetaKVStore :exec\nINSERT INTO meta_kv (key, value)\nVALUES ($1, $2)\nON CONFLICT (key) DO UPDATE\nSET value = excluded.value;\n\n\nasdsadsad\n\nыывфыв khgk\nasdыdsf\ndsdsjdfnfmdsвтьвыаыdsfsmndf,m\nы\n").await?;

    let chars = ["s", "n", ",", "d", "f", "j", "s", "d", "f", "в"];

    for (i, c) in chars.iter().enumerate() {
        server
            .change_document(
                i as i32 + 4,
                vec![TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 20,
                            character: i as u32,
                        },
                        end: Position {
                            line: 20,
                            character: i as u32,
                        },
                    }),
                    range_length: Some(0),
                    text: c.to_string(),
                }],
            )
            .await?;
    }

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_issue_303(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server.open_document("").await?;

    let chars = [
        "c", "r", "e", "a", "t", "e", " ", "t", "a", "b", "l", "e", " ", "\"\"", "h", "e", "l",
        "l", "o",
    ];
    let mut version = 1;

    for (i, c) in chars.iter().enumerate() {
        version += 1;
        server
            .change_document(
                version,
                vec![TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 0,
                            character: i as u32,
                        },
                        end: Position {
                            line: 0,
                            character: i as u32,
                        },
                    }),
                    range_length: Some(0),
                    text: c.to_string(),
                }],
            )
            .await?;
    }

    version += 1;
    server
        .change_document(
            version,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 20,
                    },
                    end: Position {
                        line: 0,
                        character: 20,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    version += 1;
    server
        .change_document(
            version,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 20,
                    },
                    end: Position {
                        line: 0,
                        character: 21,
                    },
                }),
                range_length: Some(0),
                text: "".to_string(),
            }],
        )
        .await?;

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn multiple_projects(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    // Setup configurations
    // - test_one with db connection
    let mut conf_with_db = PartialConfiguration::init();
    conf_with_db.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("test_one/postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf_with_db).unwrap(),
    );

    // -- test_two without db connection
    let mut conf_without_db = PartialConfiguration::init();
    conf_without_db.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            disable_connection: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("test_two/postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf_without_db).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize_workspaces().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    // do the same change in both workspaces and request completions in both workspaces

    server
        .open_named_document(
            "select  from public.users;\n",
            url!("test_one/document.sql"),
            "sql",
        )
        .await?;

    server
        .change_named_document(
            url!("test_one/document.sql"),
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 7,
                    },
                    end: Position {
                        line: 0,
                        character: 7,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    let res_ws_one = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("test_one/document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 8,
                },
            },
        })
        .await?
        .unwrap();

    server
        .open_named_document(
            "select  from public.users;\n",
            url!("test_two/document.sql"),
            "sql",
        )
        .await?;

    server
        .change_named_document(
            url!("test_two/document.sql"),
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 7,
                    },
                    end: Position {
                        line: 0,
                        character: 7,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    let res_ws_two = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("test_two/document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 8,
                },
            },
        })
        .await?
        .unwrap();

    // only the first one has a db connection and should return completion items
    assert!(!match res_ws_one {
        CompletionResponse::Array(a) => a.is_empty(),
        CompletionResponse::List(l) => l.items.is_empty(),
    });
    assert!(match res_ws_two {
        CompletionResponse::Array(a) => a.is_empty(),
        CompletionResponse::List(l) => l.items.is_empty(),
    });

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn extends_config(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.extends_config_test (
                id serial primary key,
                name varchar(255) not null
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    // shared config with default db connection
    let conf_with_db = PartialConfiguration::init();
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf_with_db).unwrap(),
    );

    let relative_path = if cfg!(windows) {
        "..\\postgres-language-server.jsonc"
    } else {
        "../postgres-language-server.jsonc"
    };

    // test_one extends the shared config but sets our test db
    let mut conf_with_db = PartialConfiguration::init();
    conf_with_db.merge_with(PartialConfiguration {
        extends: Some(StringSet::from_iter([relative_path.to_string()])),
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });

    fs.insert(
        url!("test_one/postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf_with_db).unwrap(),
    );

    // test_two extends it but keeps the default one
    let mut conf_without_db = PartialConfiguration::init();
    conf_without_db.merge_with(PartialConfiguration {
        extends: Some(StringSet::from_iter([relative_path.to_string()])),
        ..Default::default()
    });
    fs.insert(
        url!("test_two/postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf_without_db).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize_workspaces().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    server
        .open_named_document(
            "select  from public.extends_config_test;\n",
            url!("test_one/document.sql"),
            "sql",
        )
        .await?;

    server
        .change_named_document(
            url!("test_one/document.sql"),
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 7,
                    },
                    end: Position {
                        line: 0,
                        character: 7,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    let res_ws_one = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("test_one/document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 8,
                },
            },
        })
        .await?
        .unwrap();

    server
        .open_named_document(
            "select  from public.users;\n",
            url!("test_two/document.sql"),
            "sql",
        )
        .await?;

    server
        .change_named_document(
            url!("test_two/document.sql"),
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 7,
                    },
                    end: Position {
                        line: 0,
                        character: 7,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    let res_ws_two = server
        .get_completion(CompletionParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url!("test_two/document.sql"),
                },
                position: Position {
                    line: 0,
                    character: 8,
                },
            },
        })
        .await?
        .unwrap();

    let items_one = match res_ws_one {
        CompletionResponse::Array(ref a) => a,
        CompletionResponse::List(ref l) => &l.items,
    };

    // test one should have our test db connection and should return the completion items for the `extends_config_test` table
    assert!(items_one.iter().any(|item| {
        item.label_details.clone().is_some_and(|details| {
            details
                .description
                .is_some_and(|desc| desc.contains("public.extends_config_test"))
        })
    }));

    let items_two = match res_ws_two {
        CompletionResponse::Array(ref a) => a,
        CompletionResponse::List(ref l) => &l.items,
    };

    // test two should not have a db connection and should not return the completion items for the `extends_config_test` table
    assert!(!items_two.iter().any(|item| {
        item.label_details.clone().is_some_and(|details| {
            details
                .description
                .is_some_and(|desc| desc.contains("public.extends_config_test"))
        })
    }));

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_multiple_content_changes_single_request(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let setup = r#"
            create table public.campaign_contact_list (
                id serial primary key,
                contact_list_id integer
            );

            create table public.contact_list (
                id serial primary key,
                name varchar(255)
            );

            create table public.journey_node_contact_list (
                id serial primary key,
                contact_list_id integer
            );
        "#;

    test_db
        .execute(setup)
        .await
        .expect("Failed to setup test database");

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    // Open document with initial content that matches the log trace
    let initial_content = r#"



ALTER TABLE ONLY "public"."campaign_contact_list"
    ADD CONSTRAINT "campaign_contact_list_contact_list_id_fkey" FOREIGN KEY ("contact_list_id") REFERENCES "public"."contact_list"("id") ON UPDATE RESTRICT ON DELETE CASCADE;
"#;

    server.open_document(initial_content).await?;

    // Apply multiple content changes in a single request, similar to the log trace
    // This simulates changing "campaign" to "journey_node" in two places simultaneously
    server
        .change_document(
            4,
            vec![
                // First change: line 4, character 27-35 (changing "campaign" to "journey_node")
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 4,
                            character: 27,
                        },
                        end: Position {
                            line: 4,
                            character: 35,
                        },
                    }),
                    range_length: Some(8),
                    text: "journey_node".to_string(),
                },
                // Second change: line 5, character 20-28 (changing "campaign" to "journey_node")
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 5,
                            character: 20,
                        },
                        end: Position {
                            line: 5,
                            character: 28,
                        },
                    }),
                    range_length: Some(8),
                    text: "journey_node".to_string(),
                },
            ],
        )
        .await?;

    // make sure there is no diagnostics
    let notification = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match receiver.next().await {
                Some(ServerNotification::PublishDiagnostics(msg)) => {
                    if msg
                        .diagnostics
                        .iter()
                        .filter(|d| {
                            d.code.as_ref().is_none_or(|c| match c {
                                lsp::NumberOrString::Number(_) => true,
                                lsp::NumberOrString::String(s) => !s.starts_with("lint/"),
                            })
                        })
                        .count()
                        > 0
                    {
                        return true;
                    }
                }
                _ => continue,
            }
        }
    })
    .await
    .is_ok();

    assert!(!notification, "did not expect diagnostics");

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_plpgsql(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();

    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    server.load_configuration().await?;

    let initial_content = r#"
create function test_organisation_id ()
    returns setof text
    language plpgsql
    security invoker
    as $$
    declre
        v_organisation_id uuid;
begin
    return next is(private.organisation_id(), v_organisation_id, 'should return organisation_id of token');
end
$$;
"#;

    server.open_document(initial_content).await?;

    let got_notification = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver.next().await {
                Some(ServerNotification::PublishDiagnostics(msg)) => {
                    if msg.diagnostics.iter().any(|d| {
                        d.message
                            .contains("Invalid statement: syntax error at or near \"declre\"")
                            && d.range
                                == Range {
                                    start: Position {
                                        line: 5,
                                        character: 9,
                                    },
                                    end: Position {
                                        line: 11,
                                        character: 0,
                                    },
                                }
                    }) {
                        return true;
                    }
                }
                _ => continue,
            }
        }
    })
    .await
    .is_ok();

    assert!(
        got_notification,
        "expected diagnostics for invalid declare statement"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[tokio::test]
async fn test_crash_on_delete_character() -> Result<()> {
    let factory = ServerFactory::default();
    let (service, client) = factory.create(None).into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server.initialize().await?;
    server.initialized().await?;

    // Open document with initial CREATE INDEX statement - exactly as in log
    let initial_content = "\n\n\n\nCREATE INDEX \"idx_analytics_read_ratio\" ON \"public\".\"message\" USING \"btree\" (\"inbox_id\", \"timestamp\") INCLUDE (\"status\") WHERE (\"is_inbound\" = false);\n";

    server.open_document(initial_content).await?;

    // Add a space after false (position 148 from the log)
    server
        .change_document(
            3,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 148,
                    },
                    end: Position {
                        line: 4,
                        character: 148,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    // Follow the exact sequence from the logfile
    // Type character by character in exact order

    // Version 4: "a" at 149
    server
        .change_document(
            4,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 149,
                    },
                    end: Position {
                        line: 4,
                        character: 149,
                    },
                }),
                range_length: Some(0),
                text: "a".to_string(),
            }],
        )
        .await?;

    // Version 5: "n" at 150
    server
        .change_document(
            5,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 150,
                    },
                    end: Position {
                        line: 4,
                        character: 150,
                    },
                }),
                range_length: Some(0),
                text: "n".to_string(),
            }],
        )
        .await?;

    // Version 6: "d" at 151
    server
        .change_document(
            6,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 151,
                    },
                    end: Position {
                        line: 4,
                        character: 151,
                    },
                }),
                range_length: Some(0),
                text: "d".to_string(),
            }],
        )
        .await?;

    // Version 7: " " at 152
    server
        .change_document(
            7,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 152,
                    },
                    end: Position {
                        line: 4,
                        character: 152,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    // Version 8: "c" at 153
    server
        .change_document(
            8,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 153,
                    },
                    end: Position {
                        line: 4,
                        character: 153,
                    },
                }),
                range_length: Some(0),
                text: "c".to_string(),
            }],
        )
        .await?;

    // Version 10: "h" at 154 and "a" at 155 (two changes in one version)
    server
        .change_document(
            10,
            vec![
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 4,
                            character: 154,
                        },
                        end: Position {
                            line: 4,
                            character: 154,
                        },
                    }),
                    range_length: Some(0),
                    text: "h".to_string(),
                },
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 4,
                            character: 155,
                        },
                        end: Position {
                            line: 4,
                            character: 155,
                        },
                    }),
                    range_length: Some(0),
                    text: "a".to_string(),
                },
            ],
        )
        .await?;

    // Version 11: "n" at 156
    server
        .change_document(
            11,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 156,
                    },
                    end: Position {
                        line: 4,
                        character: 156,
                    },
                }),
                range_length: Some(0),
                text: "n".to_string(),
            }],
        )
        .await?;

    // Version 12: "n" at 157
    server
        .change_document(
            12,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 157,
                    },
                    end: Position {
                        line: 4,
                        character: 157,
                    },
                }),
                range_length: Some(0),
                text: "n".to_string(),
            }],
        )
        .await?;

    // Version 13: "e" at 158
    server
        .change_document(
            13,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 158,
                    },
                    end: Position {
                        line: 4,
                        character: 158,
                    },
                }),
                range_length: Some(0),
                text: "e".to_string(),
            }],
        )
        .await?;

    // Version 14: "l" at 159
    server
        .change_document(
            14,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 159,
                    },
                    end: Position {
                        line: 4,
                        character: 159,
                    },
                }),
                range_length: Some(0),
                text: "l".to_string(),
            }],
        )
        .await?;

    // Version 15: "_" at 160
    server
        .change_document(
            15,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 160,
                    },
                    end: Position {
                        line: 4,
                        character: 160,
                    },
                }),
                range_length: Some(0),
                text: "_".to_string(),
            }],
        )
        .await?;

    // Version 16: "t" at 161
    server
        .change_document(
            16,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 161,
                    },
                    end: Position {
                        line: 4,
                        character: 161,
                    },
                }),
                range_length: Some(0),
                text: "t".to_string(),
            }],
        )
        .await?;

    // Version 17: "y" at 162
    server
        .change_document(
            17,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 162,
                    },
                    end: Position {
                        line: 4,
                        character: 162,
                    },
                }),
                range_length: Some(0),
                text: "y".to_string(),
            }],
        )
        .await?;

    // Version 18: "p" at 163
    server
        .change_document(
            18,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 163,
                    },
                    end: Position {
                        line: 4,
                        character: 163,
                    },
                }),
                range_length: Some(0),
                text: "p".to_string(),
            }],
        )
        .await?;

    // Version 19: "e" at 164
    server
        .change_document(
            19,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 164,
                    },
                    end: Position {
                        line: 4,
                        character: 164,
                    },
                }),
                range_length: Some(0),
                text: "e".to_string(),
            }],
        )
        .await?;

    // Version 20: " " at 165
    server
        .change_document(
            20,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 165,
                    },
                    end: Position {
                        line: 4,
                        character: 165,
                    },
                }),
                range_length: Some(0),
                text: " ".to_string(),
            }],
        )
        .await?;

    // Now we should have: "WHERE ("is_inbound" = false and channel_type )"

    // Version 21: Paste the problematic text with double single quotes
    server
        .change_document(
            21,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 166,
                    },
                    end: Position {
                        line: 4,
                        character: 166,
                    },
                }),
                range_length: Some(0),
                text: "channel_type not in (''postal'', ''sms'')".to_string(),
            }],
        )
        .await?;

    // Delete "channel_type"
    server
        .change_document(
            22,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 166,
                    },
                    end: Position {
                        line: 4,
                        character: 178,
                    },
                }),
                range_length: Some(12),
                text: "".to_string(),
            }],
        )
        .await?;

    // Delete one more character
    server
        .change_document(
            23,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 166,
                    },
                    end: Position {
                        line: 4,
                        character: 167,
                    },
                }),
                range_length: Some(1),
                text: "".to_string(),
            }],
        )
        .await?;

    // This final delete should trigger the panic
    let result = server
        .change_document(
            24,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 4,
                        character: 175,
                    },
                    end: Position {
                        line: 4,
                        character: 176,
                    },
                }),
                range_length: Some(1),
                text: "".to_string(),
            }],
        )
        .await;

    assert!(result.is_ok());

    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_fix_all_code_action(test_db: PgPool) -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            host: Some(test_db.connect_options().get_host().to_string()),
            ..Default::default()
        }),
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(SectionMarkers::Dbmate),
            ..Default::default()
        }),
        ..Default::default()
    });
    fs.insert(
        url!("postgres-language-server.jsonc")
            .to_file_path()
            .unwrap()
            .into_owned(),
        serde_json::to_string_pretty(&conf).unwrap(),
    );

    let (service, client) = factory
        .create_with_fs(None, DynRef::Owned(Box::new(fs)))
        .into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);
    let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    // The client supports versioned edits.
    server
        .initialize_with_capabilities(lsp::ClientCapabilities {
            workspace: Some(lsp::WorkspaceClientCapabilities {
                workspace_edit: Some(lsp::WorkspaceEditClientCapabilities {
                    document_changes: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await?;
    server.initialized().await?;
    server.load_configuration().await?;

    // The section runs outside a transaction, so adding `CONCURRENTLY` is a safe fix.
    let doc_url = url!("test.sql");
    server
        .open_named_document(
            "-- migrate:up transaction:false\nreindex index my_index;\n",
            doc_url.clone(),
            "sql",
        )
        .await?;

    let code_actions = async |server: &mut Server, only: Vec<lsp::CodeActionKind>| {
        let response = server
            .request::<CodeActionParams, CodeActionResponse>(
                "textDocument/codeAction",
                "_code_action",
                CodeActionParams {
                    text_document: TextDocumentIdentifier {
                        uri: doc_url.clone(),
                    },
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 0),
                    },
                    context: CodeActionContext {
                        only: Some(only),
                        ..Default::default()
                    },
                    partial_result_params: PartialResultParams::default(),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                },
            )
            .await
            .unwrap()
            .unwrap();
        response
            .into_iter()
            .filter_map(|action| match action {
                lsp::CodeActionOrCommand::CodeAction(action) => Some(action),
                lsp::CodeActionOrCommand::Command(_) => None,
            })
            .collect::<Vec<_>>()
    };

    // On save, editors ask for `source.fixAll.pgls`, or for one of its parents.
    for only in ["source.fixAll.pgls", "source.fixAll", "source"] {
        let actions = code_actions(&mut server, vec![lsp::CodeActionKind::from(only)]).await;
        let fix_all = actions
            .iter()
            .find(|action| action.title == "Apply all safe fixes")
            .unwrap_or_else(|| panic!("expected a fix-all action for {only}: {actions:#?}"));
        assert_eq!(
            fix_all.kind,
            Some(lsp::CodeActionKind::from("source.fixAll.pgls"))
        );
        let Some(lsp::DocumentChanges::Edits(changes)) = fix_all
            .edit
            .as_ref()
            .and_then(|edit| edit.document_changes.as_ref())
        else {
            panic!("expected versioned edits: {fix_all:#?}");
        };
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].text_document.uri, doc_url);
        assert_eq!(changes[0].text_document.version, Some(0));
        let [lsp::OneOf::Left(edit)] = changes[0].edits.as_slice() else {
            panic!("expected one text edit: {changes:#?}");
        };
        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.range.end.line, 1);
        let mut line = "reindex index my_index;".to_string();
        line.replace_range(
            edit.range.start.character as usize..edit.range.end.character as usize,
            &edit.new_text,
        );
        assert_eq!(line.to_lowercase(), "reindex index concurrently my_index;");
    }

    // Quick fixes don't come with the fix-all action.
    let actions = code_actions(&mut server, vec![lsp::CodeActionKind::QUICKFIX]).await;
    assert!(
        actions
            .iter()
            .all(|action| action.title != "Apply all safe fixes"),
        "{actions:#?}"
    );

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

/// Waits for the next diagnostics of the document at `uri`.
async fn wait_for_diagnostics_of(
    receiver: &mut Receiver<ServerNotification>,
    uri: &Uri,
) -> Result<Vec<lsp::Diagnostic>> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let Some(ServerNotification::PublishDiagnostics(params)) = receiver.next().await else {
                bail!("the server closed the connection");
            };
            if &params.uri == uri {
                return Ok(params.diagnostics);
            }
        }
    })
    .await
    .with_context(|| format!("timed out waiting for diagnostics of {}", uri.as_str()))?
}

/// The severities of the `banDropColumn` diagnostics.
fn ban_drop_column_severities(diagnostics: &[lsp::Diagnostic]) -> Vec<lsp::DiagnosticSeverity> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                &diagnostic.code,
                Some(lsp::NumberOrString::String(code)) if code == "lint/banDropColumn"
            )
        })
        .filter_map(|diagnostic| diagnostic.severity)
        .collect()
}

/// Opens a document that drops a column, and returns the severities of its `banDropColumn`
/// diagnostics.
async fn open_and_check(
    server: &mut Server,
    receiver: &mut Receiver<ServerNotification>,
    uri: Uri,
) -> Result<Vec<lsp::DiagnosticSeverity>> {
    server
        .open_named_document("alter table users drop column name;", uri.clone(), "sql")
        .await?;
    Ok(ban_drop_column_severities(
        &wait_for_diagnostics_of(receiver, &uri).await?,
    ))
}

/// A configuration file below the workspace root applies to the files in its directory, and the
/// server picks up configuration files that are created or deleted while it runs.
#[tokio::test]
async fn nested_configuration_files_apply_to_their_directories() -> Result<()> {
    let root = tempfile::tempdir()?;
    let write_config = |directory: &str, content: &str| -> Result<()> {
        let directory = root.path().join(directory);
        std::fs::create_dir_all(&directory)?;
        std::fs::write(directory.join(ConfigName::pgls_jsonc()), content)?;
        Ok(())
    };
    write_config("", "{}")?;
    write_config("db1", r#"{ "linter": { "enabled": false } }"#)?;
    write_config(
        "db2",
        r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
    )?;

    let factory = ServerFactory::default();
    let (service, client) = factory
        .create_with_fs(
            None,
            DynRef::Owned(Box::new(OsFileSystem::new(root.path().to_path_buf()))),
        )
        .into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server
        .initialize_with_root(Uri::from_file_path(root.path()).unwrap())
        .await?;
    server.initialized().await?;

    let document = |path: &str| Uri::from_file_path(root.path().join(path)).unwrap();

    assert_eq!(
        open_and_check(&mut server, &mut receiver, document("a.sql")).await?,
        [lsp::DiagnosticSeverity::WARNING]
    );
    assert_eq!(
        open_and_check(&mut server, &mut receiver, document("db1/a.sql")).await?,
        []
    );
    assert_eq!(
        open_and_check(&mut server, &mut receiver, document("db2/schema/a.sql")).await?,
        [lsp::DiagnosticSeverity::ERROR]
    );

    // A configuration file created while the server runs.
    write_config("db3", r#"{ "linter": { "enabled": false } }"#)?;
    server
        .notify(
            "workspace/didChangeWatchedFiles",
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: document("db3/postgres-language-server.jsonc"),
                    typ: FileChangeType::CREATED,
                }],
            },
        )
        .await?;
    drain_notifications(&mut receiver);
    // The reload can come after the document is first checked.
    server
        .open_named_document(
            "alter table users drop column name;",
            document("db3/a.sql"),
            "sql",
        )
        .await?;
    wait_for_severities(&mut receiver, &document("db3/a.sql"), &[]).await?;

    // A deleted one no longer applies, and the open documents are checked again.
    std::fs::remove_file(root.path().join("db1").join(ConfigName::pgls_jsonc()))?;
    drain_notifications(&mut receiver);
    server
        .notify(
            "workspace/didChangeWatchedFiles",
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: document("db1/postgres-language-server.jsonc"),
                    typ: FileChangeType::DELETED,
                }],
            },
        )
        .await?;
    // Diagnostics published before the deletion can still be on their way.
    let db1 = document("db1/a.sql");
    let refreshed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let diagnostics = wait_for_diagnostics_of(&mut receiver, &db1).await?;
            if !ban_drop_column_severities(&diagnostics).is_empty() {
                return Ok::<_, Error>(ban_drop_column_severities(&diagnostics));
            }
        }
    })
    .await
    .context("the diagnostics were not refreshed after deleting the configuration file")??;
    assert_eq!(refreshed, [lsp::DiagnosticSeverity::WARNING]);

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

/// Waits until the diagnostics of the document at `uri` have the given `banDropColumn` severities.
async fn wait_for_severities(
    receiver: &mut Receiver<ServerNotification>,
    uri: &Uri,
    expected: &[lsp::DiagnosticSeverity],
) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let diagnostics = wait_for_diagnostics_of(receiver, uri).await?;
            if ban_drop_column_severities(&diagnostics) == expected {
                return Ok::<_, Error>(());
            }
        }
    })
    .await
    .with_context(|| format!("the diagnostics of {} were not refreshed", uri.as_str()))?
}

/// A workspace folder added after the initialization gets its own project with its configuration,
/// and a removed one no longer applies.
#[tokio::test]
async fn workspace_folders_can_be_added_and_removed() -> Result<()> {
    let root = tempfile::tempdir()?;
    for (folder, configuration) in [
        ("one", "{}"),
        (
            "two",
            r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
        ),
    ] {
        std::fs::create_dir_all(root.path().join(folder))?;
        std::fs::write(
            root.path().join(folder).join(ConfigName::pgls_jsonc()),
            configuration,
        )?;
    }
    let folder = |name: &str| WorkspaceFolder {
        uri: Uri::from_file_path(root.path().join(name)).unwrap(),
        name: name.to_string(),
    };

    let factory = ServerFactory::default();
    let (service, client) = factory
        .create_with_fs(
            None,
            DynRef::Owned(Box::new(OsFileSystem::new(root.path().to_path_buf()))),
        )
        .into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);

    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server
        .initialize_with_folders(folder("one").uri, Some(vec![folder("one")]))
        .await?;
    server.initialized().await?;

    // Outside of every workspace folder, the project of the shallowest one applies.
    let document = Uri::from_file_path(root.path().join("two").join("a.sql")).unwrap();
    assert_eq!(
        open_and_check(&mut server, &mut receiver, document.clone()).await?,
        [lsp::DiagnosticSeverity::WARNING]
    );

    let change = |added: Vec<WorkspaceFolder>, removed: Vec<WorkspaceFolder>| {
        DidChangeWorkspaceFoldersParams {
            event: WorkspaceFoldersChangeEvent { added, removed },
        }
    };

    server
        .notify(
            "workspace/didChangeWorkspaceFolders",
            change(vec![folder("two")], vec![]),
        )
        .await?;
    wait_for_severities(&mut receiver, &document, &[lsp::DiagnosticSeverity::ERROR]).await?;

    server
        .notify(
            "workspace/didChangeWorkspaceFolders",
            change(vec![], vec![folder("two")]),
        )
        .await?;
    wait_for_severities(
        &mut receiver,
        &document,
        &[lsp::DiagnosticSeverity::WARNING],
    )
    .await?;

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

/// A burst of configuration file changes, e.g. from a `git checkout`, reloads the configuration
/// once, and a change in a folder the nested configuration search skips doesn't reload it.
#[tokio::test]
async fn configuration_changes_are_coalesced() -> Result<()> {
    let root = tempfile::tempdir()?;
    let config = root.path().join(ConfigName::pgls_jsonc());
    std::fs::write(&config, "{}")?;
    let skipped = root
        .path()
        .join("node_modules/pkg")
        .join(ConfigName::pgls_jsonc());
    std::fs::create_dir_all(skipped.parent().unwrap())?;
    std::fs::write(&skipped, r#"{ "linter": { "enabled": false } }"#)?;

    let factory = ServerFactory::default();
    let (service, client) = factory
        .create_with_fs(
            None,
            DynRef::Owned(Box::new(OsFileSystem::new(root.path().to_path_buf()))),
        )
        .into_inner();
    let (stream, sink) = client.split();
    let mut server = Server::new(service);
    let (sender, mut receiver) = channel(CHANNEL_BUFFER_SIZE);
    let reader = tokio::spawn(client_handler(stream, sink, sender));

    server
        .initialize_with_root(Uri::from_file_path(root.path()).unwrap())
        .await?;
    server.initialized().await?;

    let document = Uri::from_file_path(root.path().join("a.sql")).unwrap();
    assert_eq!(
        open_and_check(&mut server, &mut receiver, document.clone()).await?,
        [lsp::DiagnosticSeverity::WARNING]
    );

    let changed = |path: &Path| DidChangeWatchedFilesParams {
        changes: vec![FileEvent {
            uri: Uri::from_file_path(path).unwrap(),
            typ: FileChangeType::CHANGED,
        }],
    };
    // Counts the times the diagnostics of the document are refreshed within a second.
    async fn refreshes(receiver: &mut Receiver<ServerNotification>, uri: &Uri) -> usize {
        let mut refreshes = 0;
        let _ = tokio::time::timeout(Duration::from_secs(1), async {
            while let Some(ServerNotification::PublishDiagnostics(params)) = receiver.next().await {
                if &params.uri == uri {
                    refreshes += 1;
                }
            }
        })
        .await;
        refreshes
    }

    server
        .notify("workspace/didChangeWatchedFiles", changed(&skipped))
        .await?;
    assert_eq!(
        refreshes(&mut receiver, &document).await,
        0,
        "a configuration file in node_modules reloaded the configuration"
    );

    for _ in 0..5 {
        server
            .notify("workspace/didChangeWatchedFiles", changed(&config))
            .await?;
    }
    assert_eq!(refreshes(&mut receiver, &document).await, 1);

    server.shutdown().await?;
    reader.abort();
    Ok(())
}
