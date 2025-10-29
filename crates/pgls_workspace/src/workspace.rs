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
        },
        completions::{CompletionsResult, GetCompletionsParams},
        diagnostics::{PullDiagnosticsParams, PullDiagnosticsResult},
        on_hover::{OnHoverParams, OnHoverResult},
    },
};

mod client;
mod server;

pub use server::StatementId;
pub(crate) use server::document::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OpenFileParams {
    pub path: PgLSPath,
    pub content: String,
    pub version: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CloseFileParams {
    pub path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChangeFileParams {
    pub path: PgLSPath,
    pub version: i32,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IsPathIgnoredParams {
    pub pgls_path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UpdateSettingsParams {
    pub configuration: PartialConfiguration,
    pub vcs_base_path: Option<PathBuf>,
    pub gitignore_matches: Vec<String>,
    pub workspace_directory: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GetFileContentParams {
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
    pub path: Option<PathBuf>,
    pub set_as_current_workspace: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UnregisterProjectFolderParams {
    pub path: PgLSPath,
}

pub trait Workspace: Send + Sync + RefUnwindSafe {
    /// Retrieves the list of diagnostics associated to a file
    fn pull_diagnostics(
        &self,
        params: PullDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError>;

    /// Retrieves a list of available code_actions for a file/cursor_position
    fn pull_code_actions(
        &self,
        params: CodeActionsParams,
    ) -> Result<CodeActionsResult, WorkspaceError>;

    fn get_completions(
        &self,
        params: GetCompletionsParams,
    ) -> Result<CompletionsResult, WorkspaceError>;

    fn on_hover(&self, params: OnHoverParams) -> Result<OnHoverResult, WorkspaceError>;

    /// Register a possible workspace project folder. Returns the key of said project. Use this key when you want to switch to different projects.
    fn register_project_folder(
        &self,
        params: RegisterProjectFolderParams,
    ) -> Result<ProjectKey, WorkspaceError>;

    /// Unregister a workspace project folder. The settings that belong to that project are deleted.
    fn unregister_project_folder(
        &self,
        params: UnregisterProjectFolderParams,
    ) -> Result<(), WorkspaceError>;

    /// Update the global settings for this workspace
    fn update_settings(&self, params: UpdateSettingsParams) -> Result<(), WorkspaceError>;

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
    path: PgLSPath,
}

impl<'app, W: Workspace + ?Sized> FileGuard<'app, W> {
    pub fn open(workspace: &'app W, params: OpenFileParams) -> Result<Self, WorkspaceError> {
        let path = params.path.clone();
        workspace.open_file(params)?;
        Ok(Self { workspace, path })
    }

    pub fn change_file(&self, version: i32, content: String) -> Result<(), WorkspaceError> {
        self.workspace.change_file(ChangeFileParams {
            path: self.path.clone(),
            version,
            content,
        })
    }

    pub fn get_file_content(&self) -> Result<String, WorkspaceError> {
        self.workspace.get_file_content(GetFileContentParams {
            path: self.path.clone(),
        })
    }

    pub fn pull_diagnostics(
        &self,
        categories: RuleCategories,
        max_diagnostics: u32,
        only: Vec<RuleSelector>,
        skip: Vec<RuleSelector>,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        self.workspace.pull_diagnostics(PullDiagnosticsParams {
            path: self.path.clone(),
            categories,
            max_diagnostics: max_diagnostics.into(),
            only,
            skip,
        })
    }
}

impl<W: Workspace + ?Sized> Drop for FileGuard<'_, W> {
    fn drop(&mut self) {
        self.workspace
            .close_file(CloseFileParams {
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

#[cfg(feature = "schema")]
impl JsonSchema for ProjectKey {
    fn schema_name() -> String {
        "ProjectKey".to_string()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <String>::json_schema(generator)
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
