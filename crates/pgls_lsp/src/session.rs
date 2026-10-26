use crate::adapters::{PositionEncoding, WideEncoding, negotiated_encoding};
use crate::configuration::{ClientOverrides, merge_layers};
use crate::diagnostics::LspError;
use crate::documents::Document;
use crate::utils;
use anyhow::Result;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use pgls_analyse::RuleCategoriesBuilder;
use pgls_configuration::{ConfigurationPathHint, PartialConfiguration};
use pgls_diagnostics::{DiagnosticExt, Error};
use pgls_fs::{ConfigName, FileSystem, PgLSPath};
use pgls_workspace::PartialConfigurationExt;
use pgls_workspace::Workspace;
use pgls_workspace::configuration::{
    LoadedConfiguration, NestedConfiguration, apply_nested_configurations, load_configuration,
};
use pgls_workspace::features;
use pgls_workspace::workspace::{
    ProjectKey, RegisterProjectFolderParams, UnregisterProjectFolderParams, UpdateSettingsParams,
};
use pgls_workspace::{DynRef, WorkspaceError};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::sync::OnceCell;
use tokio::task::AbortHandle;
use tower_lsp_server::ls_types::Uri;
use tower_lsp_server::ls_types::{self, ClientCapabilities};
use tower_lsp_server::ls_types::{MessageType, Registration};
use tower_lsp_server::ls_types::{Unregistration, WorkspaceFolder};
use tracing::{debug, error, info};

/// Debounce delay for `textDocument/didChange` diagnostics.
///
/// After each keystroke the server waits this long before running analysis.
/// If another change arrives within the window the previous task is cancelled
/// and the timer resets, so only one analysis run fires per burst.
const DIAGNOSTIC_DEBOUNCE: Duration = Duration::from_millis(300);

pub(crate) struct ClientInformation {
    #[allow(dead_code)]
    /// The name of the client
    pub(crate) name: String,

    #[allow(dead_code)]
    /// The version of the client
    pub(crate) version: Option<String>,
}

/// Key, uniquely identifying a LSP session.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub(crate) struct SessionKey(pub u64);

/// Represents the state of an LSP server session.
pub(crate) struct Session {
    /// The unique key identifying this session.
    pub(crate) key: SessionKey,

    /// The LSP client for this session.
    pub(crate) client: tower_lsp_server::Client,

    /// The parameters provided by the client in the "initialize" request
    initialize_params: OnceCell<InitializeParams>,

    pub(crate) workspace: Arc<dyn Workspace>,

    configuration_status: AtomicU8,

    /// A flag to notify a message to the user when the configuration is broken, and the LSP attempts
    /// to update the diagnostics
    notified_broken_configuration: AtomicBool,

    /// A flag to notify a message to the user when they are using the deprecated config filename
    notified_deprecated_config: AtomicBool,

    /// Whether the user was told about deprecated settings in the configuration.
    notified_deprecated_settings: AtomicBool,

    /// File system to read files inside the workspace
    pub(crate) fs: DynRef<'static, dyn FileSystem>,

    documents: RwLock<FxHashMap<ls_types::Uri, Document>>,

    pub(crate) cancellation: Arc<Notify>,

    pub(crate) config_path: Option<PathBuf>,

    /// Extra configuration from environment variables, applied on every config load.
    env_config: Option<PartialConfiguration>,

    /// Layer set through workspace/didChangeConfiguration and the custom override request.
    client_overrides: RwLock<ClientOverrides>,

    /// The workspace folders, which the client can change after the initialization. `None` if the
    /// client has none and uses the root URI.
    workspace_folders: RwLock<Option<Vec<PathBuf>>>,

    /// The projects of the workspace by their folder, deepest first: one per workspace folder, or
    /// one for the root. A document belongs to the project of the deepest folder that contains it.
    projects: RwLock<Vec<(PathBuf, ProjectKey)>>,

    /// Serialises a whole configuration transition: storing the layer, reloading the settings and
    /// refreshing the diagnostics of all open documents.
    ///
    /// Two transitions really can run at the same time — the server runs on a multi-threaded
    /// runtime and tower-lsp handles several messages concurrently — and that matters for two
    /// things. An older transition's diagnostics refresh could publish after a newer one, leaving the client with diagnostics computed under superseded settings; the staleness
    /// check does not catch that, because the document version is unchanged. And a burst of client
    /// requests would otherwise run one full reload and one type check pass per message.
    ///
    /// What it is *not* needed for is the applied layer itself: that is read immediately before the
    /// synchronous `update_settings` call, so the transition that writes last always writes the
    /// layer that was current at that moment.
    configuration_transition: tokio::sync::Mutex<()>,

    /// Configuration transitions currently running, and the highest count observed. The lock above
    /// keeps both at one; [`Session::max_concurrent_transitions`] lets the tests assert that.
    #[cfg(test)]
    running_transitions: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    max_running_transitions: std::sync::atomic::AtomicUsize,

    /// Per-URL abort handles for pending debounced diagnostic tasks.
    ///
    /// When `did_change` fires, any in-flight handle for the URL is aborted
    /// before a new task is spawned. This ensures that only one analysis run
    /// fires at the end of a rapid-fire burst rather than one per keystroke.
    diagnostic_debounce: Arc<Mutex<FxHashMap<ls_types::Uri, AbortHandle>>>,
}

/// The parameters provided by the client in the "initialize" request
struct InitializeParams {
    /// The capabilities provided by the client as part of [`ls_types::InitializeParams`]
    client_capabilities: ls_types::ClientCapabilities,
    #[allow(dead_code)]
    client_information: Option<ClientInformation>,
    root_uri: Option<Uri>,
    #[allow(unused)]
    workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[repr(u8)]
pub(crate) enum ConfigurationStatus {
    /// The configuration file was properly loaded
    Loaded = 0,
    /// The configuration file does not exist
    Missing = 1,
    /// The configuration file exists but could not be loaded
    Error = 2,
    /// Currently loading the configuration
    Loading = 3,
}

impl ConfigurationStatus {
    pub(crate) const fn is_error(&self) -> bool {
        matches!(self, ConfigurationStatus::Error)
    }
}

impl TryFrom<u8> for ConfigurationStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Loaded),
            1 => Ok(Self::Missing),
            2 => Ok(Self::Error),
            3 => Ok(Self::Loading),
            _ => Err(()),
        }
    }
}

pub(crate) type SessionHandle = Arc<Session>;

/// Holds the set of capabilities supported by the Language Server
/// instance and whether they are enabled or not
#[derive(Default)]
pub(crate) struct CapabilitySet {
    registry: FxHashMap<&'static str, (&'static str, CapabilityStatus)>,
}

/// Represents whether a capability is enabled or not, optionally holding the
/// configuration associated with the capability
pub(crate) enum CapabilityStatus {
    Enable(Option<Value>),
    Disable,
}

impl CapabilitySet {
    /// Insert a capability in the set
    pub(crate) fn add_capability(
        &mut self,
        id: &'static str,
        method: &'static str,
        status: CapabilityStatus,
    ) {
        self.registry.insert(id, (method, status));
    }
}

impl Session {
    pub(crate) fn new(
        key: SessionKey,
        client: tower_lsp_server::Client,
        workspace: Arc<dyn Workspace>,
        cancellation: Arc<Notify>,
        fs: DynRef<'static, dyn FileSystem>,
        env_config: Option<PartialConfiguration>,
    ) -> Self {
        let documents = Default::default();
        Self {
            key,
            client,
            initialize_params: OnceCell::default(),
            workspace,
            configuration_status: AtomicU8::new(ConfigurationStatus::Missing as u8),
            documents,
            fs,
            cancellation,
            config_path: None,
            notified_broken_configuration: AtomicBool::new(false),
            notified_deprecated_config: AtomicBool::new(false),
            notified_deprecated_settings: AtomicBool::new(false),
            env_config,
            client_overrides: RwLock::new(ClientOverrides::default()),
            workspace_folders: RwLock::default(),
            projects: RwLock::default(),
            configuration_transition: tokio::sync::Mutex::new(()),
            #[cfg(test)]
            running_transitions: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            max_running_transitions: std::sync::atomic::AtomicUsize::new(0),
            diagnostic_debounce: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    pub(crate) fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
    }

    /// Initialize this session instance with the incoming initialization parameters from the client
    pub(crate) fn initialize(
        &self,
        client_capabilities: ls_types::ClientCapabilities,
        client_information: Option<ClientInformation>,
        root_uri: Option<Uri>,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
    ) {
        *self.workspace_folders.write().unwrap() = workspace_folders.as_ref().map(|folders| {
            folders
                .iter()
                .filter_map(|folder| {
                    let path = folder.uri.to_file_path().map(|path| path.into_owned());
                    if path.is_none() {
                        error!(
                            "The Workspace root URI {:?} could not be parsed as a filesystem path",
                            folder.uri
                        );
                    }
                    path
                })
                .collect()
        });
        let result = self.initialize_params.set(InitializeParams {
            client_capabilities,
            client_information,
            root_uri,
            workspace_folders,
        });

        if let Err(err) = result {
            error!("Failed to initialize session: {err}");
        }
    }

    /// Register a set of capabilities with the client
    pub(crate) async fn register_capabilities(&self, capabilities: CapabilitySet) {
        let mut registrations = Vec::new();
        let mut unregistrations = Vec::new();

        let mut register_methods = String::new();
        let mut unregister_methods = String::new();

        for (id, (method, status)) in capabilities.registry {
            unregistrations.push(Unregistration {
                id: id.to_string(),
                method: method.to_string(),
            });

            if !unregister_methods.is_empty() {
                unregister_methods.push_str(", ");
            }

            unregister_methods.push_str(method);

            if let CapabilityStatus::Enable(register_options) = status {
                registrations.push(Registration {
                    id: id.to_string(),
                    method: method.to_string(),
                    register_options,
                });

                if !register_methods.is_empty() {
                    register_methods.push_str(", ");
                }

                register_methods.push_str(method);
            }
        }

        match self.client.unregister_capability(unregistrations).await {
            Err(e) => {
                error!(
                    "Error unregistering {unregister_methods:?} capabilities: {}",
                    e
                );
            }
            _ => {
                info!("Unregister capabilities {unregister_methods:?}");
            }
        }

        match self.client.register_capability(registrations).await {
            Err(e) => {
                error!("Error registering {register_methods:?} capabilities: {}", e);
            }
            _ => {
                info!("Register capabilities {register_methods:?}");
            }
        }
    }

    /// Computes diagnostics for the file matching the provided url and publishes
    /// them to the client. Called from [`handlers::text_document`] when a file's
    /// contents changes.
    #[tracing::instrument(level = "trace", skip_all, fields(url = display(url.as_str()), diagnostic_count), err)]
    pub(crate) async fn update_diagnostics(&self, url: ls_types::Uri) -> Result<(), LspError> {
        let pgls_path = self.file_path(&url)?;
        let doc = self.document(&url)?;
        let captured_version = doc.version;
        if self.configuration_status().is_error() && !self.notified_broken_configuration() {
            self.set_notified_broken_configuration();
            self.client
                    .show_message(MessageType::WARNING, "The configuration file has errors. PgLSP will report only parsing errors until the configuration is fixed.")
                    .await;
        }

        let categories = RuleCategoriesBuilder::default().all();

        let diagnostics: Vec<ls_types::Diagnostic> = {
            let result = self.workspace.pull_file_diagnostics(
                features::diagnostics::PullFileDiagnosticsParams {
                    project_key: self.project_key(&pgls_path),
                    path: pgls_path.clone(),
                    max_diagnostics: u32::MAX,
                    categories: categories.build(),
                    only: Vec::new(),
                    skip: Vec::new(),
                },
            )?;

            result
                .diagnostics
                .into_iter()
                .filter_map(|d| {
                    match utils::diagnostic_to_lsp(
                        d,
                        &url,
                        &doc.line_index,
                        self.position_encoding(),
                        None,
                    ) {
                        Ok(diag) => Some(diag),
                        Err(err) => {
                            error!("failed to convert diagnostic to LSP: {err:?}");
                            None
                        }
                    }
                })
                .collect()
        };

        let current_version = match self.document(&url) {
            Ok(doc) => doc.version,
            Err(_) => return Ok(()),
        };
        if is_stale_diagnostics(captured_version, current_version) {
            debug!(
                "discarding stale diagnostics for {}: computed for version {captured_version}, current is {current_version}",
                url.as_str()
            );
            return Ok(());
        }

        self.client
            .publish_diagnostics(url, diagnostics, Some(captured_version))
            .await;

        Ok(())
    }

    /// Schedules a debounced diagnostic update for the given URL.
    ///
    /// If a diagnostic task for `url` is already pending it is aborted first,
    /// resetting the debounce window. A new task is spawned that waits
    /// [`DIAGNOSTIC_DEBOUNCE`] before calling [`Self::update_diagnostics`].
    /// This prevents a rapid-fire `textDocument/didChange` burst from queuing
    /// one analysis run per keystroke.
    pub(crate) fn schedule_diagnostics(self: &Arc<Self>, url: ls_types::Uri) {
        // Cancel any pending task for this URL.
        if let Some(handle) = self.diagnostic_debounce.lock().unwrap().remove(&url) {
            handle.abort();
        }

        let session = Arc::clone(self);
        let url_clone = url.clone();
        let join_handle = tokio::spawn(async move {
            tokio::time::sleep(DIAGNOSTIC_DEBOUNCE).await;
            // Remove our own entry so stale handles don't accumulate.
            session
                .diagnostic_debounce
                .lock()
                .unwrap()
                .remove(&url_clone);
            if let Err(err) = session.update_diagnostics(url_clone).await {
                error!("Failed to update diagnostics: {}", err);
            }
        });

        self.diagnostic_debounce
            .lock()
            .unwrap()
            .insert(url, join_handle.abort_handle());
    }

    /// Cancels any pending debounced diagnostic task for `url`.
    ///
    /// Called from `did_close` to ensure no diagnostics are published after
    /// the document has been closed.
    pub(crate) fn cancel_pending_diagnostics(&self, url: &ls_types::Uri) {
        if let Some(handle) = self.diagnostic_debounce.lock().unwrap().remove(url) {
            handle.abort();
        }
    }

    /// Updates diagnostics for every [`Document`] in this [`Session`]
    pub(crate) async fn update_all_diagnostics(&self) {
        let mut futures: FuturesUnordered<_> = self
            .documents
            .read()
            .unwrap()
            .keys()
            .map(|url| self.update_diagnostics(url.clone()))
            .collect();

        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                error!("Error while updating diagnostics: {}", e);
            }
        }
    }

    /// Get a [`Document`] matching the provided [`ls_types::Uri`]
    ///
    /// If document does not exist, result is [WorkspaceError::NotFound]
    pub(crate) fn document(&self, url: &ls_types::Uri) -> Result<Document, Error> {
        self.documents
            .read()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| WorkspaceError::not_found().with_file_path(url.to_string()))
    }

    /// Set the [`Document`] for the provided [`ls_types::Uri`]
    ///
    /// Used by [`handlers::text_document] to synchronize documents with the client.
    pub(crate) fn insert_document(&self, url: ls_types::Uri, document: Document) {
        self.documents.write().unwrap().insert(url, document);
    }

    /// Remove the [`Document`] matching the provided [`ls_types::Uri`]
    pub(crate) fn remove_document(&self, url: &ls_types::Uri) {
        self.documents.write().unwrap().remove(url);
    }

    pub(crate) fn file_path(&self, url: &ls_types::Uri) -> Result<PgLSPath> {
        let path_to_file = match url.to_file_path() {
            None => {
                // If we can't create a path, it's probably because the file doesn't exist.
                // It can be a newly created file that it's not on disk
                PathBuf::from(url.path().as_str())
            }
            Some(path) => path.into_owned(),
        };

        Ok(PgLSPath::new(path_to_file))
    }

    /// True if the client supports dynamic registration of "workspace/didChangeConfiguration" requests
    pub(crate) fn can_register_did_change_configuration(&self) -> bool {
        self.initialize_params
            .get()
            .and_then(|c| c.client_capabilities.workspace.as_ref())
            .and_then(|c| c.did_change_configuration)
            .and_then(|c| c.dynamic_registration)
            == Some(true)
    }

    /// Get the current workspace folders
    pub(crate) fn get_workspace_folders(&self) -> Option<Vec<PathBuf>> {
        self.workspace_folders.read().unwrap().clone()
    }

    /// Applies a change of the workspace folders. The caller reloads the configuration.
    pub(crate) fn change_workspace_folders(&self, added: Vec<PathBuf>, removed: &[PathBuf]) {
        let mut folders = self.workspace_folders.write().unwrap();
        // A client without workspace folders at the start used the root URI.
        let folders = folders.get_or_insert_with(|| self.base_path().into_iter().collect());
        folders.retain(|folder| !removed.contains(folder));
        for folder in added {
            if !folders.contains(&folder) {
                folders.push(folder);
            }
        }
    }

    /// The project that the file at `path` belongs to: the one of the deepest workspace folder
    /// that contains it, or else the one of the shallowest folder. Before the configuration is loaded, the key of no
    /// project, for which the workspace only reports what needs no settings.
    pub(crate) fn project_key(&self, path: &Path) -> ProjectKey {
        let projects = self.projects.read().unwrap();
        projects
            .iter()
            .find(|(folder, _)| path.starts_with(folder))
            .or_else(|| projects.last())
            .map(|(_, key)| *key)
            .unwrap_or_default()
    }

    /// Returns the base path of the workspace on the filesystem if it has one
    pub(crate) fn base_path(&self) -> Option<PathBuf> {
        let initialize_params = self.initialize_params.get()?;

        let root_uri = initialize_params.root_uri.as_ref()?;
        match root_uri.to_file_path() {
            Some(base_path) => Some(base_path.into_owned()),
            None => {
                error!(
                    "The Workspace root URI {root_uri:?} could not be parsed as a filesystem path"
                );
                None
            }
        }
    }

    /// Returns a reference to the client capabilities for this session
    pub(crate) fn client_capabilities(&self) -> Option<&ClientCapabilities> {
        self.initialize_params
            .get()
            .map(|params| &params.client_capabilities)
    }

    /// This function attempts to read the `postgres-language-server.jsonc` configuration file of
    /// every workspace folder, or of the root URI, and update the settings of its project
    /// accordingly. The configuration files below a folder apply to the files below them.
    #[tracing::instrument(level = "trace", skip(self))]
    async fn load_workspace_settings(&self) {
        let roots = if let Some(config_path) = &self.config_path {
            // Providing a custom configuration path will not allow to support workspaces
            vec![ConfigurationPathHint::FromUser(config_path.clone())]
        } else if let Some(folders) = self.get_workspace_folders() {
            info!("Detected workspace folder.");
            folders
                .into_iter()
                .map(ConfigurationPathHint::FromWorkspace)
                .collect()
        } else {
            vec![match self.base_path() {
                None => ConfigurationPathHint::default(),
                Some(path) => ConfigurationPathHint::FromLsp(path),
            }]
        };

        self.set_configuration_status(ConfigurationStatus::Loading);
        let mut status = ConfigurationStatus::Loaded;
        let mut projects = Vec::new();
        for root in roots {
            let (project, project_status) = self.load_project(root).await;
            projects.extend(project);
            if !matches!(project_status, ConfigurationStatus::Loaded) {
                status = project_status;
            }
        }

        // Unregister the projects of the workspace folders that were removed.
        let previous = std::mem::take(&mut *self.projects.write().unwrap());
        for (folder, _) in previous {
            if !projects.iter().any(|(project, _)| project == &folder) {
                let result =
                    self.workspace
                        .unregister_project_folder(UnregisterProjectFolderParams {
                            path: PgLSPath::new(&folder),
                        });
                if let Err(error) = result {
                    error!("Failed to unregister the project folder: {}", error);
                }
            }
        }

        projects.sort_by_key(|(folder, _)| std::cmp::Reverse(folder.components().count()));
        *self.projects.write().unwrap() = projects;
        self.set_configuration_status(status);
    }

    /// Loads the configuration of a project and its nested configuration files. Returns the folder
    /// and the key of the project, unless it could not be registered.
    async fn load_project(
        &self,
        base_path: ConfigurationPathHint,
    ) -> (Option<(PathBuf, ProjectKey)>, ConfigurationStatus) {
        // The configuration files below a workspace root apply to their directories. With a
        // configuration file chosen by the user, that file applies to everything.
        let root = match &base_path {
            ConfigurationPathHint::FromWorkspace(path) | ConfigurationPathHint::FromLsp(path) => {
                Some(path.clone())
            }
            ConfigurationPathHint::FromUser(_) | ConfigurationPathHint::None => None,
        };
        let folder = root.clone().or_else(|| self.fs.working_directory());

        let project_key =
            match self
                .workspace
                .register_project_folder(RegisterProjectFolderParams {
                    path: folder.clone(),
                }) {
                Ok(project_key) => project_key,
                Err(error) => {
                    error!("Failed to register the project folder: {}", error);
                    self.client.log_message(MessageType::ERROR, &error).await;
                    return (None, ConfigurationStatus::Error);
                }
            };
        let folder = folder.unwrap_or_default();
        let is_new = !self
            .projects
            .read()
            .unwrap()
            .iter()
            .any(|(project, _)| project == &folder);

        let status = self
            .load_pgls_configuration_file(project_key, base_path)
            .await;
        if !matches!(status, ConfigurationStatus::Loaded) {
            // A project keeps its settings while its configuration is broken, and a new one is
            // only registered once its configuration loads.
            if is_new {
                let _ = self
                    .workspace
                    .unregister_project_folder(UnregisterProjectFolderParams {
                        path: PgLSPath::new(&folder),
                    });
                return (None, status);
            }
            return (Some((folder, project_key)), status);
        }
        let project = Some((folder, project_key));

        let status = match root {
            Some(root) => {
                self.load_nested_configuration_files(project_key, &root)
                    .await
            }
            None => ConfigurationStatus::Loaded,
        };
        (project, status)
    }

    /// Lays the layers of the session over a configuration file: the environment and the client
    /// overrides.
    fn configure(&self) -> impl Fn(PartialConfiguration) -> PartialConfiguration + '_ {
        let (client_overrides, overridden_sections) = {
            let overrides = self.client_overrides.read().unwrap();
            (overrides.configuration().cloned(), overrides.sections())
        };
        move |configuration| {
            let mut configuration = merge_layers(
                configuration,
                self.env_config.as_ref(),
                client_overrides.as_ref(),
            );
            overridden_sections.materialize_defaults(&mut configuration);
            configuration
        }
    }

    /// Applies every configuration file below `root` to the project, with the same layers as the
    /// root configuration.
    async fn load_nested_configuration_files(
        &self,
        project_key: ProjectKey,
        root: &Path,
    ) -> ConfigurationStatus {
        let nested = match apply_nested_configurations(
            &*self.workspace,
            &self.fs,
            project_key,
            root,
            self.configure(),
        ) {
            Ok(nested) => nested,
            Err(error) => {
                error!("Failed to apply the nested configuration files: {}", error);
                self.client.log_message(MessageType::ERROR, &error).await;
                return ConfigurationStatus::Error;
            }
        };

        let mut status = ConfigurationStatus::Loaded;
        for NestedConfiguration { directory, result } in nested {
            match result {
                Ok(()) => info!("Loaded the nested configuration in {}", directory.display()),
                Err(error) => {
                    error!(
                        "Couldn't load the configuration file in {}, reason:\n {}",
                        directory.display(),
                        error
                    );
                    self.client.log_message(MessageType::ERROR, &error).await;
                    status = ConfigurationStatus::Error;
                }
            }
        }
        status
    }

    /// Loads the configuration file at or above the root of a project into its root settings.
    async fn load_pgls_configuration_file(
        &self,
        project_key: ProjectKey,
        base_path: ConfigurationPathHint,
    ) -> ConfigurationStatus {
        match load_configuration(&self.fs, base_path.clone()) {
            Ok(loaded_configuration) => {
                let LoadedConfiguration {
                    configuration: fs_configuration,
                    directory_path: configuration_path,
                    file_path: configuration_file_path,
                    ..
                } = loaded_configuration;

                info!(
                    "Configuration loaded successfully from disk at {}",
                    configuration_file_path
                        .as_ref()
                        .map_or("<unknown>".to_string(), |p| p.display().to_string())
                );

                // Check if using deprecated config filename and notify user (once)
                if !self.notified_deprecated_config.load(Ordering::Relaxed)
                    && let Some(config_path) = &configuration_file_path
                    && let Some(file_name) = config_path.file_name().and_then(|n| n.to_str())
                    && ConfigName::is_deprecated(file_name)
                {
                    self.client
                                    .show_message(
                                        MessageType::WARNING,
                                        "You are using the deprecated config filename 'postgrestools.jsonc'. \
                                        Please rename it to 'postgres-language-server.jsonc'. \
                                        Support for the old filename will be removed in a future version."
                                    )
                                    .await;
                    self.notified_deprecated_config
                        .store(true, Ordering::Relaxed);
                }

                let deprecations = fs_configuration.deprecations();
                if !deprecations.is_empty()
                    && !self
                        .notified_deprecated_settings
                        .swap(true, Ordering::Relaxed)
                {
                    self.client
                        .show_message(
                            MessageType::WARNING,
                            format!(
                                "Your configuration uses deprecated settings:\n{}",
                                deprecations.join("\n")
                            ),
                        )
                        .await;
                }

                info!("Update workspace settings.");

                let configuration = self.configure()(fs_configuration);
                let result = configuration
                    .retrieve_gitignore_matches(&self.fs, configuration_path.as_deref());

                match result {
                    Ok((vcs_base_path, gitignore_matches)) => {
                        let result = self.workspace.update_settings(UpdateSettingsParams {
                            project_key,
                            workspace_directory: self.fs.working_directory(),
                            configuration,
                            vcs_base_path,
                            gitignore_matches,
                        });

                        if let Err(error) = result {
                            error!("Failed to set workspace settings: {}", error);
                            self.client.log_message(MessageType::ERROR, &error).await;
                            ConfigurationStatus::Error
                        } else {
                            ConfigurationStatus::Loaded
                        }
                    }
                    Err(err) => {
                        error!("Couldn't load the configuration file, reason:\n {}", err);
                        self.client.log_message(MessageType::ERROR, &err).await;
                        ConfigurationStatus::Error
                    }
                }
            }
            Err(err) => {
                error!("Couldn't load the configuration file, reason:\n {}", err);
                self.client.log_message(MessageType::ERROR, &err).await;
                ConfigurationStatus::Error
            }
        }
    }

    /// Replaces the sticky client configuration override layer and applies it. `None` clears it.
    ///
    /// Storing the layer, reloading the workspace settings and refreshing the diagnostics of all
    /// open documents happen as one transition under `configuration_transition`, so that two
    /// concurrent calls cannot interleave.
    pub(crate) async fn set_client_overrides(
        &self,
        overrides: Option<PartialConfiguration>,
    ) -> ConfigurationStatus {
        let _transition = self.configuration_transition.lock().await;
        #[cfg(test)]
        let _running = self.enter_transition();
        self.client_overrides.write().unwrap().set(overrides);
        // Everything from here on is synchronous work behind `.await`s that complete immediately, so
        // a transition only ever suspends when something makes the server wait. Yielding here in
        // tests turns that into a reliable suspension point, which is what lets the test suite
        // observe whether two transitions can overlap.
        #[cfg(test)]
        tokio::task::yield_now().await;
        self.load_workspace_settings().await;
        self.update_all_diagnostics().await;
        self.configuration_status()
    }

    /// Whether a client override layer is currently set.
    #[cfg(test)]
    pub(crate) fn has_client_overrides(&self) -> bool {
        self.client_overrides
            .read()
            .unwrap()
            .configuration()
            .is_some()
    }

    /// Reloads the configuration from disk, re-applying the sticky layers.
    ///
    /// Diagnostics are left alone: the callers decide whether the reload should refresh them.
    pub(crate) async fn reload_workspace_settings(&self) -> ConfigurationStatus {
        let _transition = self.configuration_transition.lock().await;
        #[cfg(test)]
        let _running = self.enter_transition();
        self.load_workspace_settings().await;
        self.configuration_status()
    }

    #[cfg(test)]
    fn enter_transition(&self) -> RunningTransition<'_> {
        use std::sync::atomic::AtomicUsize;

        let running = self.running_transitions.fetch_add(1, Ordering::Relaxed) + 1;
        self.max_running_transitions
            .fetch_max(running, Ordering::Relaxed);
        RunningTransition {
            running: &self.running_transitions as &AtomicUsize,
        }
    }

    /// The highest number of configuration transitions that ran at the same time.
    #[cfg(test)]
    pub(crate) fn max_concurrent_transitions(&self) -> usize {
        self.max_running_transitions.load(Ordering::Relaxed)
    }

    /// Broadcast a shutdown signal to all active connections
    pub(crate) fn broadcast_shutdown(&self) {
        self.cancellation.notify_one();
    }

    /// Retrieves information regarding the configuration status
    pub(crate) fn configuration_status(&self) -> ConfigurationStatus {
        self.configuration_status
            .load(Ordering::Relaxed)
            .try_into()
            .unwrap()
    }

    /// Updates the status of the configuration
    fn set_configuration_status(&self, status: ConfigurationStatus) {
        self.notified_broken_configuration
            .store(false, Ordering::Relaxed);
        self.configuration_status
            .store(status as u8, Ordering::Relaxed);
    }

    fn notified_broken_configuration(&self) -> bool {
        self.notified_broken_configuration.load(Ordering::Relaxed)
    }
    fn set_notified_broken_configuration(&self) {
        self.notified_broken_configuration
            .store(true, Ordering::Relaxed);
    }

    pub fn position_encoding(&self) -> PositionEncoding {
        self.initialize_params
            .get()
            .map_or(PositionEncoding::Wide(WideEncoding::Utf16), |params| {
                negotiated_encoding(&params.client_capabilities)
            })
    }
}

/// Tracks one running configuration transition for [`Session::max_concurrent_transitions`].
#[cfg(test)]
pub(crate) struct RunningTransition<'a> {
    running: &'a std::sync::atomic::AtomicUsize,
}

#[cfg(test)]
impl Drop for RunningTransition<'_> {
    fn drop(&mut self) {
        self.running.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Returns `true` when diagnostics computed for `captured_version` no longer
/// match the document's `current_version` and must therefore be discarded.
///
/// Any mismatch is treated as stale, not just a strictly newer version: a
/// document that is closed and reopened may have its version reset to a lower
/// number, so an in-flight run captured at a higher version must not publish
/// over the freshly reopened document.
fn is_stale_diagnostics(captured_version: i32, current_version: i32) -> bool {
    captured_version != current_version
}

#[cfg(test)]
mod tests {
    use super::is_stale_diagnostics;

    #[test]
    fn stale_diagnostics_are_detected_by_version() {
        // A newer version arrived while diagnostics were computed.
        assert!(is_stale_diagnostics(1, 2));
        // Same version: the result still matches the live document.
        assert!(!is_stale_diagnostics(2, 2));
        // Reopened document reset to a lower version: still stale, the
        // captured result no longer matches the live document.
        assert!(is_stale_diagnostics(5, 3));
    }
}
