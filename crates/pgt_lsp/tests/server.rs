use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use anyhow::bail;
use biome_deserialize::Merge;
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
use pgt_test_utils::test_database::get_new_test_db;
use pgt_workspace::DynRef;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::{from_value, to_value};
use sqlx::Executor;
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
    async fn open_named_document(&mut self, text: impl Display, document_name: Url) -> Result<()> {
        self.notify(
            "textDocument/didOpen",
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: document_name,
                    language_id: String::from("sql"),
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

    async fn change_document(
        &mut self,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        self.notify(
            "textDocument/didChange",
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: url!("document.sql"),
                    version,
                },
                content_changes,
            },
        )
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

#[tokio::test]
async fn test_database_connection() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

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

#[tokio::test]
async fn test_completions() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

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

#[tokio::test]
async fn test_issue_271() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

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

#[tokio::test]
async fn test_execute_statement() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

    let database = test_db
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    let host = test_db.connect_options().get_host().to_string();

    let conf = PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(database),
            host: Some(host),
            ..Default::default()
        }),
        ..Default::default()
    };

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
        .open_named_document(doc_content.to_string(), doc_url.clone())
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

#[tokio::test]
async fn test_issue_281() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

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

#[tokio::test]
async fn test_issue_303() -> Result<()> {
    let factory = ServerFactory::default();
    let mut fs = MemoryFileSystem::default();
    let test_db = get_new_test_db().await;

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
