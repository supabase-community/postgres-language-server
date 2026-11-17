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
    features::diagnostics::{PullDiagnosticsResult, PullFileDiagnosticsParams},
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
    /// The folder that relative paths and globs of the configuration are resolved against: the
    /// folder of the configuration file.
    pub workspace_directory: Option<PathBuf>,
    /// Whether the configuration replaces the root settings. Otherwise it is merged into them, and
    /// the sections it doesn't set keep their values.
    #[serde(default)]
    pub replace: bool,
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
    /// The folder with symlinks resolved, if that differs from `path`, e.g. for a folder opened
    /// through a symlink. The paths below it belong to the project too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub real_path: Option<PathBuf>,
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
pub struct FindProjectParams {
    pub path: PgLSPath,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UnregisterProjectFolderParams {
    pub path: PgLSPath,
}

/// The schemas that `invalidate_schema_cache` clears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SchemaCacheScope {
    /// The schemas loaded from database connections. A schema loaded from JSON stays.
    Connections,
    /// Every schema, including the one loaded from JSON.
    All,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct InvalidateSchemaCacheParams {
    pub scope: SchemaCacheScope,
}

/// The requests of [Workspace]: every method except [Workspace::server_info]. This list is the
/// only one. It calls `$callback!` with one `fn name(Params) -> Result;` per method, after the
/// tokens in brackets if any, and expands to:
/// - the [Workspace] trait;
/// - the [WorkspaceClient], which sends each method as the request `pgls/<name>`;
/// - the registration of the `pgls/<name>` requests in `pgls_lsp`;
/// - the TypeScript bindings of `backend-jsonrpc`, through `workspace_types::methods`.
#[macro_export]
macro_rules! for_each_workspace_method {
    ($callback:ident $([$($args:tt)*])?) => {
        $callback! {
            $([$($args)*])?

            /// Retrieves the list of diagnostics associated to a file
            fn pull_file_diagnostics($crate::features::diagnostics::PullFileDiagnosticsParams)
                -> $crate::features::diagnostics::PullDiagnosticsResult;

            /// Formats a file and returns per-statement results
            fn pull_file_formatting($crate::features::format::PullFileFormattingParams)
                -> $crate::features::format::PullFormattingResult;

            /// Retrieves the list of diagnostics associated to a database schema
            fn pull_db_diagnostics($crate::features::diagnostics::PullDatabaseDiagnosticsParams)
                -> $crate::features::diagnostics::PullDiagnosticsResult;

            /// Connects to the configured database and loads its schema, which the database-backed
            /// checks need. Reports why that failed, including for a connection that failed recently
            /// and is waiting to be retried. Reports nothing when no database is configured or the
            /// connection is disabled.
            fn check_database_connection(
                $crate::features::diagnostics::CheckDatabaseConnectionParams
            ) -> $crate::features::diagnostics::CheckDatabaseConnectionResult;

            /// Retrieves a list of available code_actions for a file/cursor_position
            fn pull_code_actions($crate::features::code_actions::CodeActionsParams)
                -> $crate::features::code_actions::CodeActionsResult;

            /// Applies the fixes of the lint rules to a file and returns its new content. The file in
            /// the workspace is not changed.
            fn fix_file($crate::features::code_actions::FixFileParams)
                -> $crate::features::code_actions::FixFileResult;

            fn get_completions($crate::features::completions::GetCompletionsParams)
                -> $crate::features::completions::CompletionsResult;

            fn on_hover($crate::features::on_hover::OnHoverParams)
                -> $crate::features::on_hover::OnHoverResult;

            /// The inlay hints of the statements that overlap a range, like the names of the
            /// parameters that function arguments are passed to.
            fn pull_inlay_hints($crate::features::inlay_hints::PullInlayHintsParams)
                -> $crate::features::inlay_hints::PullInlayHintsResult;

            /// The semantic tokens of the names that resolve to exactly one object, like tables,
            /// columns, and functions, in the statements that overlap a range or the whole file.
            fn pull_semantic_tokens($crate::features::semantic_tokens::PullSemanticTokensParams)
                -> $crate::features::semantic_tokens::PullSemanticTokensResult;

            /// Register a project folder, or return the key of the project already registered for it.
            /// Every request about a file names the project that the file belongs to.
            fn register_project_folder($crate::workspace::RegisterProjectFolderParams)
                -> $crate::workspace::ProjectKey;

            /// The project that the file at `path` belongs to: the one of the deepest folder that contains
            /// it. A project without a folder contains every path. `None` if no project contains it.
            fn find_project($crate::workspace::FindProjectParams)
                -> Option<$crate::workspace::ProjectKey>;

            /// Unregister a project folder. Its settings, including the nested ones, are deleted.
            fn unregister_project_folder($crate::workspace::UnregisterProjectFolderParams) -> ();

            /// Update the root settings of a project
            fn update_settings($crate::workspace::UpdateSettingsParams) -> ();

            /// Replace the settings of the configuration files below the root of a project. A file uses
            /// the settings of the deepest of them that contains it, or else the root settings.
            fn update_nested_settings($crate::workspace::UpdateNestedSettingsParams)
                -> $crate::workspace::UpdateNestedSettingsResult;

            /// Add a new file to the workspace
            fn open_file($crate::workspace::OpenFileParams) -> ();

            /// Remove a file from the workspace
            fn close_file($crate::workspace::CloseFileParams) -> ();

            /// Change the content of an open file
            fn change_file($crate::workspace::ChangeFileParams) -> ();

            /// Return the content of a file
            fn get_file_content($crate::workspace::GetFileContentParams) -> String;

            /// Checks if the current path is ignored by the workspace.
            ///
            /// Takes as input the path of the file that workspace is currently processing and
            /// a list of paths to match against.
            ///
            /// If the file path matches, then `true` is returned, and it should be considered ignored.
            fn is_path_ignored($crate::workspace::IsPathIgnoredParams) -> bool;

            fn execute_statement($crate::features::code_actions::ExecuteStatementParams)
                -> $crate::features::code_actions::ExecuteStatementResult;

            /// Cancels the execution of the statement, if it is running. The execution then returns the
            /// error that Postgres raises for the cancelled statement.
            fn cancel_statement_execution($crate::features::code_actions::ExecuteStatementParams)
                -> ();

            /// Invalidate the schema cache. The schema will be reloaded lazily on the next operation
            /// that requires it.
            fn invalidate_schema_cache($crate::workspace::InvalidateSchemaCacheParams) -> ();
        }
    };
}

macro_rules! workspace_trait {
    ($($(#[$attr:meta])* fn $name:ident($params:ty) -> $result:ty;)*) => {
        pub trait Workspace: Send + Sync + RefUnwindSafe {
            $(
                $(#[$attr])*
                fn $name(&self, params: $params) -> Result<$result, WorkspaceError>;
            )*

            /// Returns information about the server this workspace is connected to or `None` if the workspace isn't connected to a server.
            fn server_info(&self) -> Option<&ServerInfo>;
        }
    };
}

crate::for_each_workspace_method!(workspace_trait);

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
