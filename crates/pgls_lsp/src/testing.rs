//! A client for tests that talks to the language server in the same process: [Server] sends the
//! client's messages, and [Client] answers the server's.
//!
//! The tests of this crate use it, and so can other crates' with the `testing` feature.

// A test should fail loudly on what it didn't expect.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::any::type_name;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Error, Result, bail};
use futures::channel::mpsc::Sender;
use futures::future::BoxFuture;
use futures::{FutureExt, Sink, SinkExt, Stream, StreamExt};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value, json, to_value};
use tower::timeout::Timeout;
use tower::{Service, ServiceExt};
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc::{self, Request, Response};
use tower_lsp_server::ls_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, FileChangeType, FileEvent, InitializeParams, InitializeResult,
    InitializedParams, PublishDiagnosticsParams, ShowMessageParams, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
    WorkspaceFolder,
};

use crate::LSPServer;

/// The client's side of a connection: it sends requests and notifications to the server.
pub struct Server {
    service: Timeout<LspService<LSPServer>>,
}

impl Server {
    /// Talks to `service`, and fails a request that doesn't answer within 30 seconds.
    pub fn new(service: LspService<LSPServer>) -> Self {
        // The handlers run their work on the blocking thread pool, so the timeout measures wall
        // time, including diagnostics against the database while the other tests run.
        Self::with_timeout(service, Duration::from_secs(30))
    }

    /// Talks to `service`, and fails a request that doesn't answer within `timeout`, on the clock
    /// of the runtime: on a paused clock, that is virtual time.
    pub fn with_timeout(service: LspService<LSPServer>, timeout: Duration) -> Self {
        Self {
            service: Timeout::new(service, timeout),
        }
    }

    /// The URI of `path` in the workspace of the tests. The path is valid for the operating
    /// system, but doesn't have to exist on the host.
    pub fn uri(path: &str) -> Uri {
        let root = if cfg!(windows) {
            "file:///z%3A/workspace/"
        } else {
            "file:///workspace/"
        };
        Uri::from_str(&format!("{root}{path}")).expect("the test URIs are valid")
    }

    pub async fn notify<P>(&mut self, method: &'static str, params: P) -> Result<()>
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

    pub async fn request<P, R>(
        &mut self,
        method: &'static str,
        id: &'static str,
        params: P,
    ) -> Result<Option<R>>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.send_request(method, id, params)
            .await?
            .await?
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

    /// Sends a request and returns its response as a future, so that the client can send other
    /// messages, like `$/cancelRequest`, before it awaits the response.
    pub async fn send_request<P>(
        &mut self,
        method: &'static str,
        id: &'static str,
        params: P,
    ) -> Result<BoxFuture<'static, Result<Option<Response>>>>
    where
        P: Serialize,
    {
        let response = self
            .service
            .ready()
            .await
            .map_err(Error::msg)
            .context("ready() returned an error")?
            .call(
                Request::build(method)
                    .id(id)
                    .params(to_value(&params).context("failed to serialize params")?)
                    .finish(),
            );
        Ok(async move {
            response
                .await
                .map_err(Error::msg)
                .context("call() returned an error")
        }
        .boxed())
    }

    /// Asks the server to cancel the request `id`.
    pub async fn cancel_request(&mut self, id: &'static str) -> Result<()> {
        self.notify("$/cancelRequest", json!({ "id": id })).await
    }

    /// Basic implementation of the `initialize` request for tests
    pub async fn initialize(&mut self) -> Result<()> {
        self.initialize_with_root(Self::uri("")).await
    }

    /// Like [Server::initialize], with the given client capabilities
    pub async fn initialize_with_capabilities(
        &mut self,
        capabilities: ClientCapabilities,
    ) -> Result<()> {
        self.initialize_with_options(Some(Self::uri("")), None, capabilities)
            .await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri`
    pub async fn initialize_with_root(&mut self, root_uri: Uri) -> Result<()> {
        self.initialize_with_folders(root_uri, None).await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri` and the given workspace
    /// folders
    pub async fn initialize_with_folders(
        &mut self,
        root_uri: Uri,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
    ) -> Result<()> {
        self.initialize_with_options(
            Some(root_uri),
            workspace_folders,
            ClientCapabilities::default(),
        )
        .await
    }

    /// Like [Server::initialize], with the workspace root at `root_uri`, the given workspace
    /// folders and the given client capabilities
    // The `root_path` field is deprecated, but we still need to specify it
    #[allow(deprecated)]
    pub async fn initialize_with_options(
        &mut self,
        root_uri: Option<Uri>,
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
                    root_uri,
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
    pub async fn initialize_workspaces(&mut self) -> Result<()> {
        let _res: InitializeResult = self
            .request(
                "initialize",
                "_init",
                InitializeParams {
                    process_id: None,
                    root_path: None,
                    root_uri: Some(Self::uri("/")),
                    initialization_options: None,
                    capabilities: ClientCapabilities::default(),
                    trace: None,
                    workspace_folders: Some(vec![
                        WorkspaceFolder {
                            name: "test_one".to_string(),
                            uri: Self::uri("test_one"),
                        },
                        WorkspaceFolder {
                            name: "test_two".to_string(),
                            uri: Self::uri("test_two"),
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
    pub async fn initialized(&mut self) -> Result<()> {
        self.notify("initialized", InitializedParams {}).await
    }

    /// Basic implementation of the `shutdown` notification for tests
    pub async fn shutdown(&mut self) -> Result<()> {
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

    pub async fn open_document(&mut self, text: impl Display) -> Result<()> {
        self.notify(
            "textDocument/didOpen",
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Self::uri("document.sql"),
                    language_id: String::from("sql"),
                    version: 0,
                    text: text.to_string(),
                },
            },
        )
        .await
    }

    /// Opens a document with given contents and given name. The name must contain the extension too
    pub async fn open_named_document(
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
    pub async fn load_configuration(&mut self) -> Result<()> {
        self.notify(
            "workspace/didChangeConfiguration",
            DidChangeConfigurationParams {
                settings: to_value(()).unwrap(),
            },
        )
        .await
    }

    pub async fn change_named_document(
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

    pub async fn change_document(
        &mut self,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        self.change_named_document(Self::uri("document.sql"), version, content_changes)
            .await
    }

    pub async fn close_document(&mut self) -> Result<()> {
        self.close_named_document(Self::uri("document.sql")).await
    }

    /// Closes the document `uri`.
    pub async fn close_named_document(&mut self, uri: Uri) -> Result<()> {
        self.notify(
            "textDocument/didClose",
            DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
            },
        )
        .await
    }

    pub async fn get_completion(
        &mut self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        self.request::<CompletionParams, CompletionResponse>(
            "textDocument/completion",
            "_get_completion",
            params,
        )
        .await
    }

    /// Basic implementation of the `pgls/shutdown` request for tests
    pub async fn pgls_shutdown(&mut self) -> Result<()> {
        self.request::<_, ()>("pgls/shutdown", "_pgls_shutdown", ())
            .await?
            .context("pgls/shutdown returned None")?;
        Ok(())
    }

    /// Sets or clears the sticky client configuration overrides
    pub async fn set_configuration_overrides(&mut self, params: Value) -> Result<()> {
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
    pub async fn change_watched_configuration_file(&mut self) -> Result<()> {
        self.notify(
            "workspace/didChangeWatchedFiles",
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: Self::uri("postgres-language-server.jsonc"),
                    typ: FileChangeType::CHANGED,
                }],
            },
        )
        .await
    }

    /// Replaces the whole content of `document.sql`, which schedules a fresh diagnostics run
    pub async fn replace_document(&mut self, version: i32, text: impl Display) -> Result<()> {
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

/// What the server sent the client, as [Client] forwards it.
#[derive(Debug, PartialEq, Eq)]
pub enum ServerNotification {
    PublishDiagnostics(PublishDiagnosticsParams),
    ShowMessage(ShowMessageParams),
    /// A `workspace/*/refresh` request, by its method.
    Refresh(String),
}

/// The editor's side of a connection: it answers the requests of the server, and forwards its
/// notifications.
pub struct Client;

impl Client {
    /// Number of notifications buffered by the server-to-client channel before it starts blocking
    /// the current task
    pub const CHANNEL_BUFFER_SIZE: usize = 8;

    /// Forwards the diagnostics, messages and refresh requests of the server to `notify`, and
    /// answers every other request of the server with "method not found", until the server
    /// closes `stream`.
    pub async fn run<I, O>(
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

            if req.method() == "window/showMessage" {
                let params = req.params().expect("invalid request");
                let message = from_value(params.clone()).expect("invalid params");
                match notify.send(ServerNotification::ShowMessage(message)).await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }

            let id = match req.id() {
                Some(id) => id,
                None => continue,
            };

            if matches!(
                req.method(),
                "workspace/inlayHint/refresh" | "workspace/semanticTokens/refresh"
            ) {
                sink.send(Response::from_ok(id.clone(), Value::Null))
                    .await
                    .ok();
                let notification = ServerNotification::Refresh(req.method().to_string());
                match notify.send(notification).await {
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }

            let res = Response::from_error(id.clone(), jsonrpc::Error::method_not_found());

            sink.send(res).await.ok();
        }

        Ok(())
    }
}
