use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use anyhow::bail;
use biome_deserialize::Merge;
use biome_deserialize::StringSet;
use futures::Sink;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use futures::channel::mpsc::{Sender, channel};
use pgt_configuration::PartialConfiguration;
use pgt_configuration::database::PartialDatabaseConfiguration;
use pgt_fs::MemoryFileSystem;
use pgt_lsp::LSPServer;
use pgt_lsp::ServerFactory;
use pgt_workspace::DynRef;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::{from_value, to_value};
use sqlx::Executor;
use sqlx::PgPool;
use std::any::type_name;
use std::fmt::Display;
use std::time::Duration;
use tower::timeout::Timeout;
use tower::{Service, ServiceExt};
use tower_lsp::LspService;
use tower_lsp::jsonrpc;
use tower_lsp::jsonrpc::Response;
use tower_lsp::lsp_types as lsp;
use tower_lsp::lsp_types::CodeActionContext;
use tower_lsp::lsp_types::CodeActionParams;
use tower_lsp::lsp_types::CodeActionResponse;
use tower_lsp::lsp_types::CompletionParams;
use tower_lsp::lsp_types::CompletionResponse;
use tower_lsp::lsp_types::ExecuteCommandParams;
use tower_lsp::lsp_types::PartialResultParams;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::TextDocumentPositionParams;
use tower_lsp::lsp_types::WorkDoneProgressParams;
use tower_lsp::lsp_types::WorkspaceFolder;
use tower_lsp::lsp_types::{
    ClientCapabilities, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeResult, InitializedParams,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, Url, VersionedTextDocumentIdentifier,
};
use tower_lsp::{jsonrpc::Request, lsp_types::InitializeParams};

/// Statically build an [Url] instance that points to the file at `$path`
/// within the workspace. The filesystem path contained in the return URI is
/// guaranteed to be a valid path for the underlying operating system, but
/// doesn't have to refer to an existing file on the host machine.
macro_rules! url {
    ($path:literal) => {
        if cfg!(windows) {
            lsp::Url::parse(concat!("file:///z%3A/workspace/", $path)).unwrap()
        } else {
            lsp::Url::parse(concat!("file:///workspace/", $path)).unwrap()
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
                    bail!("shutdown returned {:?}", res)
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
    // The `root_path` field is deprecated, but we still need to specify it
    #[allow(deprecated)]
    async fn initialize(&mut self) -> Result<()> {
        let _res: InitializeResult = self
            .request(
                "initialize",
                "_init",
                InitializeParams {
                    process_id: None,
                    root_path: None,
                    root_uri: Some(url!("")),
                    initialization_options: None,
                    capabilities: ClientCapabilities::default(),
                    trace: None,
                    workspace_folders: None,
                    client_info: None,
                    locale: None,
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
                    bail!("shutdown returned {:?}", res)
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
        document_name: Url,
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
        uri: Url,
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
        params: tower_lsp::lsp_types::CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        self.request::<tower_lsp::lsp_types::CompletionParams, CompletionResponse>(
            "textDocument/completion",
            "_get_completion",
            params,
        )
        .await
    }

    /// Basic implementation of the `pgt/shutdown` request for tests
    async fn pgt_shutdown(&mut self) -> Result<()> {
        self.request::<_, ()>("pgt/shutdown", "_pgt_shutdown", ())
            .await?
            .context("pgt/shutdown returned None")?;
        Ok(())
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
    // are not exported from `tower_lsp` and cannot be named in the signature
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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

    // in this test, we want to ensure a database connection is established and the schema cache is
    // loaded. This is the case when the server sends typecheck diagnostics for the query above.
    // so we wait for diagnostics to be sent.
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver.next().await {
                Some(ServerNotification::PublishDiagnostics(msg)) => {
                    if msg
                        .diagnostics
                        .iter()
                        .any(|d| d.message.contains("column \"unknown\" does not exist"))
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

    assert!(notification, "expected diagnostics for unknown column");

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
    server.pgt_shutdown().await?;

    cancellation.await;

    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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

    assert!(res.is_some());

    server.shutdown().await?;
    reader.abort();

    Ok(())
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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
        let result = sqlx::query!(
            r#"
            select exists (
                select 1 as exists
                from pg_catalog.pg_tables
                where tablename = 'users'
            );
        "#
        )
        .fetch_one(&test_db.clone())
        .await;

        result.unwrap().exists.unwrap()
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
                if command.is_some_and(|cmd| &cmd.command == "pgt.executeStatement") {
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("test_one/postgrestools.jsonc").to_file_path().unwrap(),
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
        url!("test_two/postgrestools.jsonc").to_file_path().unwrap(),
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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
        url!("postgrestools.jsonc").to_file_path().unwrap(),
        serde_json::to_string_pretty(&conf_with_db).unwrap(),
    );

    let relative_path = if cfg!(windows) {
        "..\\postgrestools.jsonc"
    } else {
        "../postgrestools.jsonc"
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
        url!("test_one/postgrestools.jsonc").to_file_path().unwrap(),
        serde_json::to_string_pretty(&conf_with_db).unwrap(),
    );

    // test_two extends it but keeps the default one
    let mut conf_without_db = PartialConfiguration::init();
    conf_without_db.merge_with(PartialConfiguration {
        extends: Some(StringSet::from_iter([relative_path.to_string()])),
        ..Default::default()
    });
    fs.insert(
        url!("test_two/postgrestools.jsonc").to_file_path().unwrap(),
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

// └─┐pgt_lsp::handlers::text_document::did_open{params=DidOpenTextDocumentParams { text_document: TextDocumentItem { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, language_id: "sql", version: 0, text: "alter table \"public\".\"event\" drop constraint \"assign_event\";\n\nalter table \"public\".\"event\" drop constraint \"unassign_event\";\n\nalter table \"public\".\"event\" drop constraint \"validate_entities\";\n\ndrop index if exists \"public\".\"idx_appointment_custom_field_string\";\n\ndrop index if exists \"public\".\"idx_contact_custom_field_string\";\n\ndrop index if exists \"public\".\"idx_deal_custom_field_string\";\n\nCREATE INDEX idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);\n\nCREATE INDEX idx_contact_custom_field_string_value_btree ON public.contact_custom_field USING btree (custom_field_id, contact_id, string_value) WHERE (type = 'string'::field_type);\n\nCREATE INDEX idx_deal_custom_field_string_value_btree ON public.deal_custom_field USING btree (custom_field_id, deal_id, string_value) WHERE (type = 'string'::field_type);\n\nalter table \"public\".\"event\" add constraint \"assign_event\" CHECK (((type <> 'assign'::event_type) OR ((source_type = ANY (ARRAY['employee'::event_entity_type, 'api_token'::event_entity_type, 'service_role'::event_entity_type, 'rule'::event_entity_type, 'cs_agent_session'::event_entity_type])) AND (target_type = ANY (ARRAY['employee'::event_entity_type, 'cs_agent'::event_entity_type]))))) not valid;\n\nalter table \"public\".\"event\" validate constraint \"assign_event\";\n\nalter table \"public\".\"event\" add constraint \"unassign_event\" CHECK (((type <> 'unassign'::event_type) OR ((source_type = ANY (ARRAY['employee'::event_entity_type, 'api_token'::event_entity_type, 'service_role'::event_entity_type, 'rule'::event_entity_type, 'cs_agent_session'::event_entity_type])) AND (target_type IS NULL)))) not valid;\n\nalter table \"public\".\"event\" validate constraint \"unassign_event\";\n\nalter table \"public\".\"event\" add constraint \"validate_entities\" CHECK (((((source_type IS NOT NULL) AND (source_type = 'comment'::event_entity_type)) = (source_comment_id IS NOT NULL)) AND (((source_type IS NOT NULL) AND (source_type = 'channel'::event_entity_type)) = (source_channel_id IS NOT NULL)) AND (((source_type IS NOT NULL) AND (source_type = 'recipient'::event_entity_type)) = (source_recipient_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'employee'::event_entity_type)) = (target_employee_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'cs_agent'::event_entity_type)) = (target_cs_agent_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'comment'::event_entity_type)) = (target_comment_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'message'::event_entity_type)) = (target_message_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'tag'::event_entity_type)) = (target_tag_id IS NOT NULL)) AND (((target_type IS NOT NULL) AND (target_type = 'message_action'::event_entity_type)) = (target_message_action_id IS NOT NULL)))) not valid;\n\nalter table \"public\".\"event\" validate constraint \"validate_entities\";\n\n\n" } }}
//   └─┐pgt_workspace::workspace::server::open_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql"}
//   ┌─┘
//   ├─2025-06-02 9:46:46.224804  11ms INFO pgt_workspace::workspace::server Pulled 0 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 3 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 0, character: 0 }, end: Position { line: 5, character: 0 } }), range_length: Some(192), text: "" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=3}
//     ├─2025-06-02 9:46:50.316788   0ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 3 })
//     ├─2025-06-02 9:46:50.316957   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 2 })
//     ├─2025-06-02 9:46:50.317032   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 1 })
//     ├─2025-06-02 9:46:50.317102   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 0 })
//     ├─2025-06-02 9:46:50.317161   1ms DEBUG pgt_workspace::workspace::server::parsed_document Adding statement: id:Root(RootId { inner: 15 }), text:"drop index if exists \"public\".\"idx_appointment_custom_field_string\";"
//   ┌─┘
//   ├─2025-06-02 9:46:50.321136   6ms INFO pgt_workspace::workspace::server Pulled 0 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 4 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 13, character: 0 }, end: Position { line: 26, character: 0 } }), range_length: Some(2097), text: "" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=4}
//     ├─2025-06-02 9:46:53.741847   0ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 14 })
//     ├─2025-06-02 9:46:53.741989   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 13 })
//     ├─2025-06-02 9:46:53.742232   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 12 })
//     ├─2025-06-02 9:46:53.742321   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 11 })
//     ├─2025-06-02 9:46:53.742426   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 10 })
//     ├─2025-06-02 9:46:53.742493   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 9 })
//     ├─2025-06-02 9:46:53.742643   1ms DEBUG pgt_workspace::workspace::server::parsed_document Deleting statement: id Root(RootId { inner: 8 })
//     ├─2025-06-02 9:46:53.742737   1ms DEBUG pgt_workspace::workspace::server::parsed_document Adding statement: id:Root(RootId { inner: 16 }), text:"CREATE INDEX idx_deal_custom_field_string_value_btree ON public.deal_custom_field USING btree (custom_field_id, deal_id, string_value) WHERE (type = 'string'::field_type);"
//   ┌─┘
//   ├─2025-06-02 9:46:53.744504   4ms INFO pgt_workspace::workspace::server Pulled 0 diagnostic(s)
// ┌─┘
// ├─2025-06-02 9:46:54.366107  83m  WARN tower_lsp Got a textDocument/didSave notification, but it is not implemented
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 6 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 13 }, end: Position { line: 7, character: 13 } }), range_length: Some(0), text: "I" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=6}
//     ├─2025-06-02 9:47:00.147075   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 6 }) (new id Root(RootId { inner: 17 })). Range 13..13, Changed from '"CREATE INDEX idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX Iidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "I"
//   ┌─┘
//   ├─2025-06-02 9:47:00.148493   2ms INFO pgt_workspace::workspace::server Pulled 0 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 14 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: Invoked, trigger_character: None }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="214"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX Iidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="14"}
//     ┌─┘
//     ├─2025-06-02 9:47:00.201384  52ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 17
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 7 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 14 }, end: Position { line: 7, character: 14 } }), range_length: Some(0), text: "F" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=7}
//     ├─2025-06-02 9:47:00.295825   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 17 }) (new id Root(RootId { inner: 18 })). Range 14..14, Changed from '"CREATE INDEX Iidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IFidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "F"
//   ┌─┘
//   ├─2025-06-02 9:47:00.296768   1ms INFO pgt_workspace::workspace::server Pulled 0 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 8 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 15 }, end: Position { line: 7, character: 15 } }), range_length: Some(0), text: " " }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=8}
//     ├─2025-06-02 9:47:00.363926   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 18 }) (new id Root(RootId { inner: 19 })). Range 15..15, Changed from '"CREATE INDEX IFidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: " "
//   ┌─┘
//   ├─2025-06-02 9:47:00.364897   1ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 16 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: TriggerCharacter, trigger_character: Some(" ") }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="216"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX IF idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="16"}
//     ┌─┘
//     ├─2025-06-02 9:47:00.405793  40ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 19
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 9 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 16 }, end: Position { line: 7, character: 16 } }), range_length: Some(0), text: "N" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=9}
//     ├─2025-06-02 9:47:00.418329   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 19 }) (new id Root(RootId { inner: 20 })). Range 16..16, Changed from '"CREATE INDEX IF idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF Nidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "N"
//   ┌─┘
//   ├─2025-06-02 9:47:00.41892   1ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 17 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: Invoked, trigger_character: None }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="217"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX IF Nidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="17"}
//     ┌─┘
//     ├─2025-06-02 9:47:00.448103  28ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 20
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 10 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 17 }, end: Position { line: 7, character: 17 } }), range_length: Some(0), text: "O" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=10}
//     ├─2025-06-02 9:47:00.588574   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 20 }) (new id Root(RootId { inner: 21 })). Range 17..17, Changed from '"CREATE INDEX IF Nidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "O"
//   ┌─┘
//   ├─2025-06-02 9:47:00.589522   1ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 11 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 18 }, end: Position { line: 7, character: 18 } }), range_length: Some(0), text: "T" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=11}
//     ├─2025-06-02 9:47:00.721041   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 21 }) (new id Root(RootId { inner: 22 })). Range 18..18, Changed from '"CREATE INDEX IF NOidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOTidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "T"
//   ┌─┘
//   ├─2025-06-02 9:47:00.722806   2ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 12 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 19 }, end: Position { line: 7, character: 19 } }), range_length: Some(0), text: " " }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=12}
//     ├─2025-06-02 9:47:00.724899   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 22 }) (new id Root(RootId { inner: 23 })). Range 19..19, Changed from '"CREATE INDEX IF NOTidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: " "
//   ┌─┘
//   ├─2025-06-02 9:47:00.725877   1ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 20 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: TriggerCharacter, trigger_character: Some(" ") }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="220"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX IF NOT idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="20"}
//     ┌─┘
//     ├─2025-06-02 9:47:00.776552  50ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 23
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 13 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 20 }, end: Position { line: 7, character: 20 } }), range_length: Some(0), text: "E" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=13}
//     ├─2025-06-02 9:47:00.813959   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 23 }) (new id Root(RootId { inner: 24 })). Range 20..20, Changed from '"CREATE INDEX IF NOT idx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT Eidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "E"
//   ┌─┘
//   ├─2025-06-02 9:47:00.814674   1ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 21 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: Invoked, trigger_character: None }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="221"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX IF NOT Eidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="21"}
//     ┌─┘
//     ├─2025-06-02 9:47:00.848701  33ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 24
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 14 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 21 }, end: Position { line: 7, character: 21 } }), range_length: Some(0), text: "X" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=14}
//     ├─2025-06-02 9:47:01.051286   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 24 }) (new id Root(RootId { inner: 25 })). Range 21..21, Changed from '"CREATE INDEX IF NOT Eidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "X"
//   ┌─┘
//   ├─2025-06-02 9:47:01.052446   2ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 15 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 22 }, end: Position { line: 7, character: 22 } }), range_length: Some(0), text: "I" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=15}
//     ├─2025-06-02 9:47:01.201943   1ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 25 }) (new id Root(RootId { inner: 26 })). Range 22..22, Changed from '"CREATE INDEX IF NOT EXidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXIidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "I"
//   ┌─┘
//   ├─2025-06-02 9:47:01.203949   3ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 16 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 23 }, end: Position { line: 7, character: 23 } }), range_length: Some(0), text: "S" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=16}
//     ├─2025-06-02 9:47:01.310226   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 26 }) (new id Root(RootId { inner: 27 })). Range 23..23, Changed from '"CREATE INDEX IF NOT EXIidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "S"
//   ┌─┘
//   ├─2025-06-02 9:47:01.311452   2ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 17 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 24 }, end: Position { line: 7, character: 24 } }), range_length: Some(0), text: "T" }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=17}
//     ├─2025-06-02 9:47:01.391161   1ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 27 }) (new id Root(RootId { inner: 28 })). Range 24..24, Changed from '"CREATE INDEX IF NOT EXISidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISTidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "T"
//   ┌─┘
//   ├─2025-06-02 9:47:01.393041   3ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 19 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 25 }, end: Position { line: 7, character: 25 } }), range_length: Some(0), text: "S" }, TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 7, character: 26 }, end: Position { line: 7, character: 26 } }), range_length: Some(0), text: " " }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=19}
//     ├─2025-06-02 9:47:01.480581   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 28 }) (new id Root(RootId { inner: 29 })). Range 25..25, Changed from '"CREATE INDEX IF NOT EXISTidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISTSidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "S"
//     ├─2025-06-02 9:47:01.480915   1ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 29 }) (new id Root(RootId { inner: 30 })). Range 27..27, Changed from '"CREATE INDEX IF NOT EXISTSidx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISTSi dx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);"', changed text: " "
//   ┌─┘
//   ├─2025-06-02 9:47:01.481901   2ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::completions::get_completions{params=CompletionParams { text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None } }, position: Position { line: 7, character: 27 } }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None }, context: Some(CompletionContext { trigger_kind: TriggerCharacter, trigger_character: Some(" ") }) }}
//   └─┐pgt_workspace::workspace::server::get_completions{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", position="227"}
//     └─┐pgt_completions::complete::complete{text="CREATE INDEX IF NOT EXISTSi dx_appointment_custom_field_string_value_btree ON public.appointment_custom_field USING btree (custom_field_id, appointment_id, string_value) WHERE (type = 'string'::field_type);", position="27"}
//     ┌─┘
//     ├─2025-06-02 9:47:01.530715  48ms DEBUG pgt_workspace::workspace::server Found 0 completion items for statement with id 30
//   ┌─┘
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 20 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 9, character: 13 }, end: Position { line: 9, character: 13 } }), range_length: Some(0), text: "IF NOT EXISTS " }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=20}
//     ├─2025-06-02 9:47:08.691711   1ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 7 }) (new id Root(RootId { inner: 31 })). Range 13..13, Changed from '"CREATE INDEX idx_contact_custom_field_string_value_btree ON public.contact_custom_field USING btree (custom_field_id, contact_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISTS idx_contact_custom_field_string_value_btree ON public.contact_custom_field USING btree (custom_field_id, contact_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "IF NOT EXISTS "
//   ┌─┘
//   ├─2025-06-02 9:47:08.693581   3ms INFO pgt_workspace::workspace::server Pulled 1 diagnostic(s)
// ┌─┘
// └─┐pgt_lsp::handlers::text_document::did_change{params=DidChangeTextDocumentParams { text_document: VersionedTextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", query: None, fragment: None }, version: 21 }, content_changes: [TextDocumentContentChangeEvent { range: Some(Range { start: Position { line: 11, character: 13 }, end: Position { line: 11, character: 13 } }), range_length: Some(0), text: "IF NOT EXISTS " }] }}
//   └─┐pgt_workspace::workspace::server::change_file{path="/Users/psteinroe/Developer/hellomateo/supabase/migrations/20250423142692_better_cf_indices.sql", version=21}
//     ├─2025-06-02 9:47:10.683595   0ms DEBUG pgt_workspace::workspace::server::parsed_document Modifying statement with id Root(RootId { inner: 16 }) (new id Root(RootId { inner: 32 })). Range 13..13, Changed from '"CREATE INDEX idx_deal_custom_field_string_value_btree ON public.deal_custom_field USING btree (custom_field_id, deal_id, string_value) WHERE (type = 'string'::field_type);"' to '"CREATE INDEX IF NOT EXISTS idx_deal_custom_field_string_value_btree ON public.deal_custom_field USING btree (custom_field_id, deal_id, string_value) WHERE (type = 'string'::field_type);"', changed text: "IF NOT EXISTS "
