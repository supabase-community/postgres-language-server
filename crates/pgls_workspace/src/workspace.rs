use std::{panic::RefUnwindSafe, path::PathBuf, sync::Arc};

pub use self::client::{TransportRequest, WorkspaceClient, WorkspaceTransport};
use pgls_analyse::RuleCategories;
use pgls_configuration::{PartialConfiguration, RuleSelector};
use pgls_fs::PgLSPath;
#[cfg(feature = "schema")]
use schemars::{JsonSchema, SchemaGenerator, schema::Schema};
use serde::{Deserialize, Serialize};
use slotmap::{DenseSlotMap, new_key_type};

use crate::{
    WorkspaceError,
    features::{
        code_actions::{
            CodeActionsParams, CodeActionsResult, ExecuteStatementParams, ExecuteStatementResult,
            FixFileParams, FixFileResult,
        },
        completions::{CompletionsResult, GetCompletionsParams},
        diagnostics::{
            CheckDatabaseConnectionParams, CheckDatabaseConnectionResult,
            PullDatabaseDiagnosticsParams, PullDiagnosticsResult, PullFileDiagnosticsParams,
        },
        format::{PullFileFormattingParams, PullFormattingResult},
        inlay_hints::{PullInlayHintsParams, PullInlayHintsResult},
        on_hover::{OnHoverParams, OnHoverResult},
    },
};

mod client;
pub mod server;

pub use server::StatementId;
pub(crate) use server::document::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OpenFileParams {
    pub project_key: ProjectKey,
    pub path: PgLSPath,
    pub content: String,
    pub version: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CloseFileParams {
    pub project_key: ProjectKey,
    pub path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChangeFileParams {
    pub project_key: ProjectKey,
    pub path: PgLSPath,
    pub version: i32,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IsPathIgnoredParams {
    pub project_key: ProjectKey,
    pub pgls_path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UpdateSettingsParams {
    /// The project whose root settings are updated.
    pub project_key: ProjectKey,
    pub configuration: PartialConfiguration,
    pub vcs_base_path: Option<PathBuf>,
    pub gitignore_matches: Vec<String>,
    pub workspace_directory: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GetFileContentParams {
    pub project_key: ProjectKey,
    pub path: PgLSPath,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ServerInfo {
    /// The name of the server as defined by the server.
    pub name: String,

    /// The server's version as defined by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RegisterProjectFolderParams {
    /// The folder of the project. A project without a folder contains every path.
    pub path: Option<PathBuf>,
}

/// The configuration file of a folder below the root of a project.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NestedSettingsParams {
    /// The folder of the configuration file. Its settings apply to the files below it.
    pub directory: PathBuf,
    pub configuration: PartialConfiguration,
    pub vcs_base_path: Option<PathBuf>,
    pub gitignore_matches: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdateNestedSettingsParams {
    pub project_key: ProjectKey,
    /// Every configuration file below the root of the project. They replace the previous ones.
    pub nested: Vec<NestedSettingsParams>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdateNestedSettingsResult {
    /// The folders whose settings now apply. The others are ignored by the settings above them,
    /// or are not below the root of the project.
    pub applied: Vec<PathBuf>,
    /// The folders whose configuration could not be applied, with the reason. Their files use
    /// the default settings, never the ones of the configuration above them.
    pub errors: Vec<NestedSettingsError>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NestedSettingsError {
    pub directory: PathBuf,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UnregisterProjectFolderParams {
    pub path: PgLSPath,
}

pub trait Workspace: Send + Sync + RefUnwindSafe {
    /// Retrieves the list of diagnostics associated to a file
    fn pull_file_diagnostics(
        &self,
        params: PullFileDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError>;

    /// Formats a file and returns per-statement results
    fn pull_file_formatting(
        &self,
        params: PullFileFormattingParams,
    ) -> Result<PullFormattingResult, WorkspaceError>;

    /// Retrieves the list of diagnostics associated to a database schema
    fn pull_db_diagnostics(
        &self,
        params: PullDatabaseDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError>;

    /// Connects to the configured database and loads its schema, which the database-backed
    /// checks need. Reports why that failed, including for a connection that failed recently
    /// and is waiting to be retried. Reports nothing when no database is configured or the
    /// connection is disabled.
    fn check_database_connection(
        &self,
        params: CheckDatabaseConnectionParams,
    ) -> Result<CheckDatabaseConnectionResult, WorkspaceError>;

    /// Retrieves a list of available code_actions for a file/cursor_position
    fn pull_code_actions(
        &self,
        params: CodeActionsParams,
    ) -> Result<CodeActionsResult, WorkspaceError>;

    /// Applies the fixes of the lint rules to a file and returns its new content. The file in
    /// the workspace is not changed.
    fn fix_file(&self, params: FixFileParams) -> Result<FixFileResult, WorkspaceError>;

    fn get_completions(
        &self,
        params: GetCompletionsParams,
    ) -> Result<CompletionsResult, WorkspaceError>;

    fn on_hover(&self, params: OnHoverParams) -> Result<OnHoverResult, WorkspaceError>;

    /// The inlay hints of the statements that overlap a range, like the names of the
    /// parameters that function arguments are passed to.
    fn pull_inlay_hints(
        &self,
        params: PullInlayHintsParams,
    ) -> Result<PullInlayHintsResult, WorkspaceError>;

    /// Register a project folder, or return the key of the project already registered for it.
    /// Every request about a file names the project that the file belongs to.
    fn register_project_folder(
        &self,
        params: RegisterProjectFolderParams,
    ) -> Result<ProjectKey, WorkspaceError>;

    /// Unregister a project folder. Its settings, including the nested ones, are deleted.
    fn unregister_project_folder(
        &self,
        params: UnregisterProjectFolderParams,
    ) -> Result<(), WorkspaceError>;

    /// Update the root settings of a project
    fn update_settings(&self, params: UpdateSettingsParams) -> Result<(), WorkspaceError>;

    /// Replace the settings of the configuration files below the root of a project. A file uses
    /// the settings of the deepest of them that contains it, or else the root settings.
    fn update_nested_settings(
        &self,
        params: UpdateNestedSettingsParams,
    ) -> Result<UpdateNestedSettingsResult, WorkspaceError>;

    /// Add a new file to the workspace
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError>;

    /// Remove a file from the workspace
    fn close_file(&self, params: CloseFileParams) -> Result<(), WorkspaceError>;

    /// Change the content of an open file
    fn change_file(&self, params: ChangeFileParams) -> Result<(), WorkspaceError>;

    /// Returns information about the server this workspace is connected to or `None` if the workspace isn't connected to a server.
    fn server_info(&self) -> Option<&ServerInfo>;

    /// Return the content of a file
    fn get_file_content(&self, params: GetFileContentParams) -> Result<String, WorkspaceError>;

    /// Checks if the current path is ignored by the workspace.
    ///
    /// Takes as input the path of the file that workspace is currently processing and
    /// a list of paths to match against.
    ///
    /// If the file path matches, then `true` is returned, and it should be considered ignored.
    fn is_path_ignored(&self, params: IsPathIgnoredParams) -> Result<bool, WorkspaceError>;

    fn execute_statement(
        &self,
        params: ExecuteStatementParams,
    ) -> Result<ExecuteStatementResult, WorkspaceError>;

    /// Invalidate the schema cache.
    ///
    /// # Arguments
    /// * `all` - If true, clears all cached schemas. If false, clears the schemas loaded from a
    ///   database connection, but keeps one set from JSON.
    ///
    /// The schema will be reloaded lazily on the next operation that requires it.
    fn invalidate_schema_cache(&self, all: bool) -> Result<(), WorkspaceError>;
}

/// Convenience function for constructing a server instance of [Workspace]
pub fn server() -> Box<dyn Workspace> {
    Box::new(server::WorkspaceServer::new())
}

/// Convenience function for constructing a server instance of [Workspace]
pub fn server_sync() -> Arc<dyn Workspace> {
    Arc::new(server::WorkspaceServer::new())
}

// Convenience function for constructing a client instance of [Workspace]
pub fn client<T>(transport: T) -> Result<Box<dyn Workspace>, WorkspaceError>
where
    T: WorkspaceTransport + RefUnwindSafe + Send + Sync + 'static,
{
    Ok(Box::new(client::WorkspaceClient::new(transport)?))
}

/// [RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
/// guard for an open file in a workspace, takes care of closing the file
/// automatically on drop
pub struct FileGuard<'app, W: Workspace + ?Sized> {
    workspace: &'app W,
    project_key: ProjectKey,
    path: PgLSPath,
}

impl<'app, W: Workspace + ?Sized> FileGuard<'app, W> {
    pub fn open(workspace: &'app W, params: OpenFileParams) -> Result<Self, WorkspaceError> {
        let project_key = params.project_key;
        let path = params.path.clone();
        workspace.open_file(params)?;
        Ok(Self {
            workspace,
            project_key,
            path,
        })
    }

    /// The project that the file belongs to
    pub fn project_key(&self) -> ProjectKey {
        self.project_key
    }

    pub fn change_file(&self, version: i32, content: String) -> Result<(), WorkspaceError> {
        self.workspace.change_file(ChangeFileParams {
            project_key: self.project_key,
            path: self.path.clone(),
            version,
            content,
        })
    }

    pub fn get_file_content(&self) -> Result<String, WorkspaceError> {
        self.workspace.get_file_content(GetFileContentParams {
            project_key: self.project_key,
            path: self.path.clone(),
        })
    }

    pub fn pull_file_diagnostics(
        &self,
        categories: RuleCategories,
        max_diagnostics: u32,
        only: Vec<RuleSelector>,
        skip: Vec<RuleSelector>,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        self.workspace
            .pull_file_diagnostics(PullFileDiagnosticsParams {
                project_key: self.project_key,
                path: self.path.clone(),
                categories,
                max_diagnostics,
                only,
                skip,
            })
    }

    pub fn pull_file_formatting(
        &self,
        params: crate::features::format::PullFileFormattingParams,
    ) -> Result<crate::features::format::PullFormattingResult, WorkspaceError> {
        self.workspace.pull_file_formatting(params)
    }
}

impl<W: Workspace + ?Sized> Drop for FileGuard<'_, W> {
    fn drop(&mut self) {
        self.workspace
            .close_file(CloseFileParams {
                project_key: self.project_key,
                path: self.path.clone(),
            })
            // `close_file` can only error if the file was already closed, in
            // this case it's generally better to silently matcher the error
            // than panic (especially in a drop handler)
            .ok();
    }
}

new_key_type! {
    pub struct ProjectKey;
}

/// Identifies a project of the workspace. [ProjectKey] is serialized like this.
#[cfg(feature = "schema")]
#[derive(JsonSchema)]
#[allow(dead_code)]
struct SerializedProjectKey {
    idx: u32,
    version: u32,
}

#[cfg(feature = "schema")]
impl JsonSchema for ProjectKey {
    fn schema_name() -> String {
        "ProjectKey".to_string()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        SerializedProjectKey::json_schema(generator)
    }
}

#[derive(Debug, Default)]
pub struct WorkspaceData<V> {
    /// [DenseSlotMap] is the slowest type in insertion/removal, but the fastest in iteration
    ///
    /// Users wouldn't change workspace folders very often,
    paths: DenseSlotMap<ProjectKey, V>,
}

impl<V> WorkspaceData<V> {
    /// Inserts an item
    pub fn insert(&mut self, item: V) -> ProjectKey {
        self.paths.insert(item)
    }

    /// Removes an item
    pub fn remove(&mut self, key: ProjectKey) {
        self.paths.remove(key);
    }

    /// Get a reference of the value
    pub fn get(&self, key: ProjectKey) -> Option<&V> {
        self.paths.get(key)
    }

    /// Get a mutable reference of the value
    pub fn get_mut(&mut self, key: ProjectKey) -> Option<&mut V> {
        self.paths.get_mut(key)
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn iter(&self) -> WorkspaceDataIterator<'_, V> {
        WorkspaceDataIterator::new(self)
    }
}

pub struct WorkspaceDataIterator<'a, V> {
    iterator: slotmap::dense::Iter<'a, ProjectKey, V>,
}

impl<'a, V> WorkspaceDataIterator<'a, V> {
    fn new(data: &'a WorkspaceData<V>) -> Self {
        Self {
            iterator: data.paths.iter(),
        }
    }
}

impl<'a, V> Iterator for WorkspaceDataIterator<'a, V> {
    type Item = (ProjectKey, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
