use crate::capabilities::server_capabilities;
use crate::handlers;
use crate::session::{
    CapabilitySet, CapabilityStatus, ClientInformation, Session, SessionHandle, SessionKey,
};
use crate::utils::{into_lsp_error, panic_to_lsp_error};
use futures::FutureExt;
use futures::future::ready;
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_fs::{ConfigName, FileSystem, OsFileSystem};
use pgls_workspace::workspace::{RegisterProjectFolderParams, UnregisterProjectFolderParams};
use pgls_workspace::{DynRef, Workspace, workspace};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Notify;
use tokio::task::spawn_blocking;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::{ClientSocket, lsp_types::*};
use tower_lsp::{LanguageServer, LspService, Server};
use tracing::{Instrument, error, info};

pub struct LSPServer {
    session: SessionHandle,
    /// Map of all sessions connected to the same [ServerFactory] as this [LSPServer].
    sessions: Sessions,
    /// If this is true the server will broadcast a shutdown signal once the
    /// last client disconnected
    stop_on_disconnect: bool,
    /// This shared flag is set to true once at least one session has been
    /// initialized on this server instance
    is_initialized: Arc<AtomicBool>,
}

impl RefUnwindSafe for LSPServer {}

impl LSPServer {
    fn new(
        session: SessionHandle,
        sessions: Sessions,
        stop_on_disconnect: bool,
        is_initialized: Arc<AtomicBool>,
    ) -> Self {
        Self {
            session,
            sessions,
            stop_on_disconnect,
            is_initialized,
        }
    }

    async fn setup_capabilities(&self) {
        let mut capabilities = CapabilitySet::default();

        capabilities.add_capability(
            "pgls_did_change_extension_settings",
            "workspace/didChangeConfiguration",
            if self.session.can_register_did_change_configuration() {
                CapabilityStatus::Enable(None)
            } else {
                CapabilityStatus::Disable
            },
        );

        capabilities.add_capability(
            "pgls_did_change_workspace_settings",
            "workspace/didChangeWatchedFiles",
            match self.session.base_path() {
                Some(base_path) => {
                    let watchers = ConfigName::file_names()
                        .iter()
                        .map(|config_name| FileSystemWatcher {
                            glob_pattern: GlobPattern::String(format!(
                                "{}/{}",
                                base_path.display(),
                                config_name
                            )),
                            kind: Some(WatchKind::all()),
                        })
                        .collect();

                    CapabilityStatus::Enable(Some(json!(
                        DidChangeWatchedFilesRegistrationOptions { watchers }
                    )))
                }
                _ => CapabilityStatus::Disable,
            },
        );

        self.session.register_capabilities(capabilities).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LSPServer {
    #[allow(deprecated)]
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(
            root_uri = params.root_uri.as_ref().map(display),
            capabilities = debug(&params.capabilities),
            client_info = params.client_info.as_ref().map(debug),
            workspace_folders = params.workspace_folders.as_ref().map(debug),
        )
    )]
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        info!("Starting Language Server...");
        self.is_initialized.store(true, Ordering::Relaxed);

        let server_capabilities = server_capabilities(&params.capabilities);

        self.session.initialize(
            params.capabilities,
            params.client_info.map(|client_info| ClientInformation {
                name: client_info.name,
                version: client_info.version,
            }),
            params.root_uri,
            params.workspace_folders,
        );

        //
        let init = InitializeResult {
            capabilities: server_capabilities,
            server_info: Some(ServerInfo {
                name: String::from(env!("CARGO_PKG_NAME")),
                version: Some(pgls_configuration::VERSION.to_string()),
            }),
        };

        Ok(init)
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn initialized(&self, params: InitializedParams) {
        let _ = params;

        info!("Attempting to load the configuration",);

        self.session.reload_workspace_settings().await;

        let msg = format!("Server initialized with PID: {}", std::process::id());
        self.session
            .client
            .log_message(MessageType::INFO, msg)
            .await;

        self.setup_capabilities().await;

        // Diagnostics are disabled by default, so update them after fetching workspace config
        self.session.update_all_diagnostics().await;
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        handlers::configuration::did_change_configuration(&self.session, params).await;
        self.setup_capabilities().await;
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let file_paths = params
            .changes
            .iter()
            .map(|change| change.uri.to_file_path());
        for file_path in file_paths {
            match file_path {
                Ok(file_path) => {
                    let base_path = self.session.base_path();
                    if let Some(base_path) = base_path {
                        let possible_config_json = file_path.strip_prefix(&base_path);
                        if let Ok(watched_file) = possible_config_json
                            && ConfigName::file_names()
                                .contains(&&*watched_file.display().to_string())
                        {
                            self.session.reload_workspace_settings().await;
                            self.setup_capabilities().await;
                            // self.session.update_all_diagnostics().await;
                            // for now we are only interested to the configuration file,
                            // so it's OK to exist the loop
                            break;
                        }
                    }
                }
                Err(_) => {
                    error!(
                        "The Workspace root URI {file_path:?} could not be parsed as a filesystem path"
                    );
                    continue;
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        handlers::text_document::did_open(&self.session, params)
            .await
            .ok();
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(e) = handlers::text_document::did_change(&self.session, params).await {
            error!("{}", e);
        };
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        handlers::text_document::did_close(&self.session, params)
            .await
            .ok();
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        for removed in &params.event.removed {
            if let Ok(project_path) = self.session.file_path(&removed.uri) {
                let result = self
                    .session
                    .workspace
                    .unregister_project_folder(UnregisterProjectFolderParams { path: project_path })
                    .map_err(into_lsp_error);

                if let Err(err) = result {
                    error!("Failed to remove project from the workspace: {}", err);
                    self.session
                        .client
                        .log_message(MessageType::ERROR, err)
                        .await;
                }
            }
        }

        for added in &params.event.added {
            if let Ok(project_path) = self.session.file_path(&added.uri) {
                let result = self
                    .session
                    .workspace
                    .register_project_folder(RegisterProjectFolderParams {
                        path: Some(project_path.to_path_buf()),
                        set_as_current_workspace: true,
                    })
                    .map_err(into_lsp_error);

                if let Err(err) = result {
                    error!("Failed to add project to the workspace: {}", err);
                    self.session
                        .client
                        .log_message(MessageType::ERROR, err)
                        .await;
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        match handlers::hover::on_hover(&self.session, params) {
            Ok(result) => LspResult::Ok(result.map(|contents| Hover {
                contents,
                range: None,
            })),
            Err(e) => LspResult::Err(into_lsp_error(e)),
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        match handlers::completions::get_completions(&self.session, params) {
            Ok(result) => LspResult::Ok(Some(result)),
            Err(e) => LspResult::Err(into_lsp_error(e)),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        match handlers::code_actions::get_actions(&self.session, params) {
            Ok(result) => {
                tracing::trace!("Got {} Code Action(s)", result.len());
                return LspResult::Ok(Some(result));
            }
            Err(e) => LspResult::Err(into_lsp_error(e)),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> LspResult<Option<serde_json::Value>> {
        match handlers::code_actions::execute_command(&self.session, params).await {
            // we'll inform the client within `code_actions::execute_command`
            Ok(_) => LspResult::Ok(None),
            Err(err) => LspResult::Err(into_lsp_error(err)),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        match handlers::formatting::formatting(&self.session, params) {
            Ok(result) => LspResult::Ok(result),
            Err(e) => LspResult::Err(into_lsp_error(e)),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        match handlers::formatting::range_formatting(&self.session, params) {
            Ok(result) => LspResult::Ok(result),
            Err(e) => LspResult::Err(into_lsp_error(e)),
        }
    }
}

impl Drop for LSPServer {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            let _removed = sessions.remove(&self.session.key);
            debug_assert!(_removed.is_some(), "Session did not exist.");

            if self.stop_on_disconnect
                && sessions.is_empty()
                && self.is_initialized.load(Ordering::Relaxed)
            {
                self.session.cancellation.notify_one();
            }
        }
    }
}

/// Map of active sessions connected to a [ServerFactory].
type Sessions = Arc<Mutex<FxHashMap<SessionKey, SessionHandle>>>;

/// Helper method for wrapping a [Workspace] method in a `custom_method` for
/// the [LSPServer]
macro_rules! workspace_method {
    ( $builder:ident, $method:ident ) => {
        $builder = $builder.custom_method(
            concat!("pgls/", stringify!($method)),
            |server: &LSPServer, params| {
                let span = tracing::trace_span!(concat!("pgls/", stringify!($method)), params = ?params).or_current();
                tracing::info!("Received request: {}", stringify!($method));

                let workspace = server.session.workspace.clone();
                let result = spawn_blocking(move || {
                    let _guard = span.entered();
                    workspace.$method(params)
                });

                result.map(move |result| {
                    // The type of `result` is `Result<Result<R, RomeError>, JoinError>`,
                    // where the inner result is the return value of `$method` while the
                    // outer one is added by `spawn_blocking` to catch panics or
                    // cancellations of the task
                    match result {
                        Ok(Ok(result)) => Ok(result),
                        Ok(Err(err)) => Err(into_lsp_error(err)),
                        Err(err) => match err.try_into_panic() {
                            Ok(err) => Err(panic_to_lsp_error(err)),
                            Err(err) => Err(into_lsp_error(err)),
                        },
                    }
                })
            },
        );
    };
}

/// Factory data structure responsible for creating [ServerConnection] handles
/// for each incoming connection accepted by the server
#[derive(Default)]
pub struct ServerFactory {
    /// Synchronization primitive used to broadcast a shutdown signal to all
    /// active connections
    cancellation: Arc<Notify>,
    /// Optional [Workspace] instance shared between all clients. Currently
    /// this field is always [None] (meaning each connection will get its own
    /// workspace) until we figure out how to handle concurrent access to the
    /// same workspace from multiple client
    workspace: Option<Arc<dyn Workspace>>,

    /// The sessions of the connected clients indexed by session key.
    sessions: Sessions,

    /// Session key generator. Stores the key of the next session.
    next_session_key: AtomicU64,

    /// If this is true the server will broadcast a shutdown signal once the
    /// last client disconnected
    stop_on_disconnect: bool,
    /// This shared flag is set to true once at least one sessions has been
    /// initialized on this server instance
    is_initialized: Arc<AtomicBool>,
    /// Configuration from environment variables (DATABASE_URL, PGHOST, etc.).
    /// Computed once at factory creation and passed to each session.
    env_config: Option<pgls_configuration::PartialConfiguration>,
}

impl ServerFactory {
    pub fn new(stop_on_disconnect: bool) -> Self {
        let env_config = PartialDatabaseConfiguration::from_env().map(|db| {
            pgls_configuration::PartialConfiguration {
                db: Some(db),
                ..Default::default()
            }
        });
        Self {
            cancellation: Arc::default(),
            workspace: None,
            sessions: Sessions::default(),
            next_session_key: AtomicU64::new(0),
            stop_on_disconnect,
            is_initialized: Arc::default(),
            env_config,
        }
    }

    pub fn create(&self, config_path: Option<PathBuf>) -> ServerConnection {
        self.create_with_fs(config_path, DynRef::Owned(Box::<OsFileSystem>::default()))
    }

    /// Create a new [ServerConnection] from this factory
    pub fn create_with_fs(
        &self,
        config_path: Option<PathBuf>,
        fs: DynRef<'static, dyn FileSystem>,
    ) -> ServerConnection {
        let workspace = self
            .workspace
            .clone()
            .unwrap_or_else(workspace::server_sync);

        let session_key = SessionKey(self.next_session_key.fetch_add(1, Ordering::Relaxed));

        let env_config = self.env_config.clone();
        let mut builder = LspService::build(move |client| {
            let mut session = Session::new(
                session_key,
                client,
                workspace,
                self.cancellation.clone(),
                fs,
                env_config,
            );
            if let Some(path) = config_path {
                session.set_config_path(path);
            }
            let handle = Arc::new(session);

            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_key, handle.clone());

            LSPServer::new(
                handle,
                self.sessions.clone(),
                self.stop_on_disconnect,
                self.is_initialized.clone(),
            )
        });

        // "shutdown" is not part of the Workspace API
        builder = builder.custom_method("pgls/shutdown", |server: &LSPServer, (): ()| {
            info!("Sending shutdown signal");
            server.session.broadcast_shutdown();
            ready(Ok(Some(())))
        });

        // Snake case matches this server's other pgls methods; pgls/setSchema is on the separate WASM surface.
        builder = builder.custom_method(
            "pgls/set_configuration_overrides",
            |server: &LSPServer,
             params: handlers::configuration::SetConfigurationOverridesParams| {
                // The payload is deliberately kept out of the span: it carries whatever the client
                // wants to override, including database credentials.
                let span = tracing::trace_span!("pgls/set_configuration_overrides").or_current();
                let session = server.session.clone();
                async move {
                    handlers::configuration::set_configuration_overrides(&session, params)
                        .await
                        .map(Some)
                }
                .instrument(span)
            },
        );

        workspace_method!(builder, is_path_ignored);
        workspace_method!(builder, update_settings);
        workspace_method!(builder, get_file_content);
        workspace_method!(builder, open_file);
        workspace_method!(builder, change_file);
        workspace_method!(builder, close_file);
        workspace_method!(builder, pull_file_diagnostics);
        workspace_method!(builder, check_database_connection);
        workspace_method!(builder, get_completions);
        workspace_method!(builder, register_project_folder);
        workspace_method!(builder, unregister_project_folder);
        workspace_method!(builder, invalidate_schema_cache);

        let (service, socket) = builder.finish();
        ServerConnection { socket, service }
    }

    /// Return a handle to the cancellation token for this server process
    pub fn cancellation(&self) -> Arc<Notify> {
        self.cancellation.clone()
    }
}

/// Handle type created by the server for each incoming connection
pub struct ServerConnection {
    socket: ClientSocket,
    service: LspService<LSPServer>,
}

impl ServerConnection {
    /// Destructure a connection into its inner service instance and socket
    pub fn into_inner(self) -> (LspService<LSPServer>, ClientSocket) {
        (self.service, self.socket)
    }

    /// Accept an incoming connection and run the server async I/O loop to
    /// completion
    pub async fn accept<I, O>(self, stdin: I, stdout: O)
    where
        I: AsyncRead + Unpin,
        O: AsyncWrite,
    {
        Server::new(stdin, stdout, self.socket)
            .serve(self.service)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Document;
    use crate::session::SessionHandle;
    use futures::StreamExt;
    use pgls_configuration::PartialConfiguration;
    use pgls_fs::{OsFileSystem, PgLSPath};
    use pgls_workspace::workspace::{IsPathIgnoredParams, OpenFileParams};
    use std::future::ready;
    use std::path::Path;
    use tower_lsp::lsp_types::Url;

    /// A session backed by a real working directory, so that reloading the configuration keeps
    /// updating the settings of the *same* project. [`pgls_fs::MemoryFileSystem`] has no working
    /// directory, which makes every reload register a fresh project with default settings and would
    /// therefore hide whether a cleared override is really reset.
    ///
    /// The caller owns the client socket so it can decide how to drain it; nothing drains it by
    /// default, and logging to a client that is never read would block a transition.
    fn session_in(
        working_directory: &Path,
    ) -> (LspService<LSPServer>, SessionHandle, ClientSocket) {
        std::fs::write(working_directory.join(ConfigName::pgls_jsonc()), "{}")
            .expect("failed to write the configuration file");

        let factory = ServerFactory::default();
        let connection = factory.create_with_fs(
            None,
            DynRef::Owned(Box::new(OsFileSystem::new(working_directory.to_path_buf()))),
        );
        let session = factory
            .sessions
            .lock()
            .unwrap()
            .values()
            .next()
            .cloned()
            .expect("the factory registers the session it creates");

        let (service, socket) = connection.into_inner();
        (service, session, socket)
    }

    /// An override layer that ignores a single file name. `files.ignore` makes the settings that
    /// the workspace actually applied observable through the public [`Workspace::is_path_ignored`],
    /// without needing a database.
    fn ignore_override() -> PartialConfiguration {
        serde_json::from_value(json!({ "files": { "ignore": ["ignored.sql"] } }))
            .expect("a valid configuration")
    }

    fn is_ignored(session: &SessionHandle, path: &Path) -> bool {
        session
            .workspace
            .is_path_ignored(IsPathIgnoredParams {
                pgls_path: PgLSPath::new(path),
            })
            .expect("is_path_ignored")
    }

    /// Clearing an override has to restore the baseline. The configuration file is `{}` and there is
    /// no environment configuration, so nothing below the override layer configures `files` — and
    /// `Settings::merge_with_configuration` keeps a section that the incoming configuration omits,
    /// which is why the cleared section has to be materialized with its defaults.
    #[tokio::test]
    async fn clearing_client_overrides_restores_the_baseline() {
        let working_directory = tempfile::tempdir().unwrap();
        let (_service, session, socket) = session_in(working_directory.path());
        let (stream, _sink) = socket.split();
        tokio::spawn(stream.for_each(|_| ready(())));
        let ignored = working_directory.path().join("ignored.sql");

        session.set_client_overrides(Some(ignore_override())).await;
        assert!(
            is_ignored(&session, &ignored),
            "the override should have been applied to the workspace"
        );

        session.set_client_overrides(None).await;
        assert!(
            !is_ignored(&session, &ignored),
            "clearing the override should have reset `files` to its defaults"
        );
    }

    /// Concurrent set/clear transitions must not interleave: each one stores the layer, reloads the
    /// workspace settings and refreshes the diagnostics of every open document, so two of them
    /// running at once could apply their steps out of order.
    ///
    /// The load-bearing assertion is that no two transitions ever run at the same time. A
    /// transition only suspends where the server has to wait for something, which is why
    /// [`Session::set_client_overrides`] yields once under `cfg(test)` — without that yield these
    /// transitions would run to completion one after another even with the lock removed.
    ///
    /// That the workspace settings agree with the session's layer afterwards holds either way,
    /// because the layer is read immediately before the settings are applied. It is asserted here
    /// as a guard for the sticky-section reset, which it does catch.
    #[tokio::test]
    async fn concurrent_client_override_changes_keep_session_and_workspace_in_sync() {
        let working_directory = tempfile::tempdir().unwrap();
        let (_service, session, socket) = session_in(working_directory.path());
        let ignored = working_directory.path().join("ignored.sql");
        let untouched = working_directory.path().join("kept.sql");

        // A transition refreshes the diagnostics of every open document, so give it one to publish.
        let document = working_directory.path().join("document.sql");
        session
            .workspace
            .open_file(OpenFileParams {
                path: PgLSPath::new(&document),
                version: 0,
                content: "select 1;".to_string(),
            })
            .expect("open_file");
        session.insert_document(
            Url::from_file_path(&document).expect("a file url"),
            Document::new(0, "select 1;"),
        );

        let (stream, _sink) = socket.split();
        tokio::spawn(stream.for_each(|_| ready(())));

        for round in 0..4 {
            let transitions: Vec<_> = (0..16)
                .map(|index| {
                    let session = session.clone();
                    let overrides = (index % 2 == 0).then(ignore_override);
                    tokio::spawn(async move { session.set_client_overrides(overrides).await })
                })
                .collect();

            for transition in transitions {
                transition.await.expect("a transition panicked");
            }

            assert_eq!(
                is_ignored(&session, &ignored),
                session.has_client_overrides(),
                "round {round}: the workspace settings disagree with the session's override layer"
            );
            assert!(
                !is_ignored(&session, &untouched),
                "round {round}: an unrelated path must never be ignored"
            );
            assert_eq!(
                session.max_concurrent_transitions(),
                1,
                "round {round}: configuration transitions overlapped"
            );
        }
    }
}
