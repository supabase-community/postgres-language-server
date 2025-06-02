use std::{
    fs,
    panic::RefUnwindSafe,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use analyser::AnalyserVisitorBuilder;
use async_helper::run_async;
use connection_manager::ConnectionManager;
use dashmap::DashMap;
use document::Document;
use futures::{StreamExt, stream};
use parsed_document::{
    AsyncDiagnosticsMapper, CursorPositionFilter, DefaultMapper, ExecuteStatementMapper,
    ParsedDocument, SyncDiagnosticsMapper,
};
use pgt_analyse::{AnalyserOptions, AnalysisFilter};
use pgt_analyser::{Analyser, AnalyserConfig, AnalyserContext};
use pgt_diagnostics::{
    Diagnostic, DiagnosticExt, Error, Severity, serde::Diagnostic as SDiagnostic,
};
use pgt_fs::{ConfigName, PgTPath};
use pgt_typecheck::{IdentifierType, TypecheckParams, TypedIdentifier};
use schema_cache_manager::SchemaCacheManager;
use sqlx::{Executor, PgPool};
use tracing::{debug, info};

use crate::{
    WorkspaceError,
    configuration::to_analyser_rules,
    features::{
        code_actions::{
            self, CodeAction, CodeActionKind, CodeActionsResult, CommandAction,
            CommandActionCategory, ExecuteStatementParams, ExecuteStatementResult,
        },
        completions::{CompletionsResult, GetCompletionsParams, get_statement_for_completions},
        diagnostics::{PullDiagnosticsParams, PullDiagnosticsResult},
    },
    settings::{WorkspaceSettings, WorkspaceSettingsHandle, WorkspaceSettingsHandleMut},
};

use super::{
    GetFileContentParams, IsPathIgnoredParams, OpenFileParams, ProjectKey,
    RegisterProjectFolderParams, ServerInfo, UnregisterProjectFolderParams, UpdateSettingsParams,
    Workspace,
};

pub use statement_identifier::StatementId;

mod analyser;
mod annotation;
mod async_helper;
mod change;
mod connection_key;
mod connection_manager;
pub(crate) mod document;
mod migration;
pub(crate) mod parsed_document;
mod pg_query;
mod schema_cache_manager;
mod sql_function;
mod statement_identifier;
mod tree_sitter;

pub(super) struct WorkspaceServer {
    /// global settings object for this workspace
    settings: RwLock<WorkspaceSettings>,

    /// Stores the schema cache for this workspace
    schema_cache: SchemaCacheManager,

    parsed_documents: DashMap<PgTPath, ParsedDocument>,

    connection: ConnectionManager,
}

/// The `Workspace` object is long-lived, so we want it to be able to cross
/// unwind boundaries.
/// In return, we have to make sure operations on the workspace either do not
/// panic, of that panicking will not result in any broken invariant (it would
/// not result in any undefined behavior as catching an unwind is safe, but it
/// could lead too hard to debug issues)
impl RefUnwindSafe for WorkspaceServer {}

impl WorkspaceServer {
    /// Create a new [Workspace]
    ///
    /// This is implemented as a crate-private method instead of using
    /// [Default] to disallow instances of [Workspace] from being created
    /// outside a [crate::App]
    pub(crate) fn new() -> Self {
        Self {
            settings: RwLock::default(),
            parsed_documents: DashMap::default(),
            schema_cache: SchemaCacheManager::new(),
            connection: ConnectionManager::new(),
        }
    }

    /// Provides a reference to the current settings
    fn workspaces(&self) -> WorkspaceSettingsHandle {
        WorkspaceSettingsHandle::new(&self.settings)
    }

    fn workspaces_mut(&self) -> WorkspaceSettingsHandleMut {
        WorkspaceSettingsHandleMut::new(&self.settings)
    }

    fn get_current_connection(&self) -> Option<PgPool> {
        let settings = self.workspaces();
        let settings = settings.settings()?;
        self.connection.get_pool(&settings.db)
    }

    /// Register a new project in the current workspace
    fn register_project(&self, path: PathBuf) -> ProjectKey {
        let mut workspace = self.workspaces_mut();
        let workspace_mut = workspace.as_mut();
        workspace_mut.insert_project(path.clone())
    }

    /// Retrieves the current project path
    fn get_current_project_path(&self) -> Option<PgTPath> {
        self.workspaces().path().cloned()
    }

    /// Sets the current project of the current workspace
    fn set_current_project(&self, project_key: ProjectKey) {
        let mut workspace = self.workspaces_mut();
        let workspace_mut = workspace.as_mut();
        workspace_mut.set_current_project(project_key);
    }

    /// Checks whether the current path belongs to the current project.
    ///
    /// If there's a match, and the match **isn't** the current project, it returns the new key.
    fn path_belongs_to_current_workspace(&self, path: &PgTPath) -> Option<ProjectKey> {
        let workspaces = self.workspaces();
        workspaces.as_ref().path_belongs_to_current_workspace(path)
    }

    fn is_ignored_by_migration_config(&self, path: &Path) -> bool {
        let settings = self.workspaces();
        let settings = settings.settings();
        let Some(settings) = settings else {
            return false;
        };
        settings
            .migrations
            .as_ref()
            .and_then(|migration_settings| {
                let ignore_before = migration_settings.after.as_ref()?;
                let migrations_dir = migration_settings.path.as_ref()?;
                let migration = migration::get_migration(path, migrations_dir)?;

                Some(&migration.sequence_number <= ignore_before)
            })
            .unwrap_or(false)
    }

    /// Check whether a file is ignored in the top-level config `files.ignore`/`files.include`
    fn is_ignored(&self, path: &Path) -> bool {
        let file_name = path.file_name().and_then(|s| s.to_str());
        // Never ignore Postgres Tools's config file regardless `include`/`ignore`
        (file_name != Some(ConfigName::pgt_jsonc())) &&
            // Apply top-level `include`/`ignore
            (self.is_ignored_by_top_level_config(path) || self.is_ignored_by_migration_config(path))
    }

    /// Check whether a file is ignored in the top-level config `files.ignore`/`files.include`
    fn is_ignored_by_top_level_config(&self, path: &Path) -> bool {
        let settings = self.workspaces();
        let settings = settings.settings();
        let Some(settings) = settings else {
            return false;
        };

        let is_included = settings.files.included_files.is_empty()
            || is_dir(path)
            || settings.files.included_files.matches_path(path);
        !is_included
            || settings.files.ignored_files.matches_path(path)
            || settings.files.git_ignore.as_ref().is_some_and(|ignore| {
                // `matched_path_or_any_parents` panics if `source` is not under the gitignore root.
                // This checks excludes absolute paths that are not a prefix of the base root.
                if !path.has_root() || path.starts_with(ignore.path()) {
                    // Because Postgres Tools passes a list of paths,
                    // we use `matched_path_or_any_parents` instead of `matched`.
                    ignore
                        .matched_path_or_any_parents(path, path.is_dir())
                        .is_ignore()
                } else {
                    false
                }
            })
    }
}

impl Workspace for WorkspaceServer {
    fn register_project_folder(
        &self,
        params: RegisterProjectFolderParams,
    ) -> Result<ProjectKey, WorkspaceError> {
        let current_project_path = self.get_current_project_path();
        debug!(
            "Compare the current project with the new one {:?} {:?} {:?}",
            current_project_path,
            params.path.as_ref(),
            current_project_path.as_deref() != params.path.as_ref()
        );

        let is_new_path = match (current_project_path.as_deref(), params.path.as_ref()) {
            (Some(current_project_path), Some(params_path)) => current_project_path != params_path,
            (Some(_), None) => {
                // If the current project is set, but no path is provided, we assume it's a new project
                true
            }
            _ => true,
        };

        if is_new_path {
            let path = params.path.unwrap_or_default();
            let key = self.register_project(path.clone());
            if params.set_as_current_workspace {
                self.set_current_project(key);
            }
            Ok(key)
        } else {
            Ok(self.workspaces().as_ref().get_current_project_key())
        }
    }

    fn unregister_project_folder(
        &self,
        params: UnregisterProjectFolderParams,
    ) -> Result<(), WorkspaceError> {
        let mut workspace = self.workspaces_mut();
        workspace.as_mut().remove_project(params.path.as_path());
        Ok(())
    }

    /// Update the global settings for this workspace
    ///
    /// ## Panics
    /// This function may panic if the internal settings mutex has been poisoned
    /// by another thread having previously panicked while holding the lock
    #[tracing::instrument(level = "trace", skip(self), err)]
    fn update_settings(&self, params: UpdateSettingsParams) -> Result<(), WorkspaceError> {
        let mut workspace = self.workspaces_mut();

        workspace
            .as_mut()
            .get_current_settings_mut()
            .merge_with_configuration(
                params.configuration,
                params.workspace_directory,
                params.vcs_base_path,
                params.gitignore_matches.as_slice(),
            )?;

        Ok(())
    }

    /// Add a new file to the workspace
    #[tracing::instrument(level = "info", skip_all, fields(path = params.path.as_path().as_os_str().to_str()), err)]
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError> {
        self.parsed_documents
            .entry(params.path.clone())
            .or_insert_with(|| {
                ParsedDocument::new(params.path.clone(), params.content, params.version)
            });

        if let Some(project_key) = self.path_belongs_to_current_workspace(&params.path) {
            self.set_current_project(project_key);
        }

        Ok(())
    }

    /// Remove a file from the workspace
    fn close_file(&self, params: super::CloseFileParams) -> Result<(), WorkspaceError> {
        self.parsed_documents
            .remove(&params.path)
            .ok_or_else(WorkspaceError::not_found)?;

        Ok(())
    }

    /// Change the content of an open file
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        version = params.version
    ), err)]
    fn change_file(&self, params: super::ChangeFileParams) -> Result<(), WorkspaceError> {
        let mut parser =
            self.parsed_documents
                .entry(params.path.clone())
                .or_insert(ParsedDocument::new(
                    params.path.clone(),
                    "".to_string(),
                    params.version,
                ));

        parser.apply_change(params);

        Ok(())
    }

    fn server_info(&self) -> Option<&ServerInfo> {
        None
    }

    fn get_file_content(&self, params: GetFileContentParams) -> Result<String, WorkspaceError> {
        let document = self
            .parsed_documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;
        Ok(document.get_document_content().to_string())
    }

    fn is_path_ignored(&self, params: IsPathIgnoredParams) -> Result<bool, WorkspaceError> {
        Ok(self.is_ignored(params.pgt_path.as_path()))
    }

    fn pull_code_actions(
        &self,
        params: code_actions::CodeActionsParams,
    ) -> Result<code_actions::CodeActionsResult, WorkspaceError> {
        let parser = self
            .parsed_documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let settings = self.workspaces();
        let settings = settings.settings();

        let disabled_reason = match settings {
            Some(settings) if settings.db.allow_statement_executions => None,
            Some(_) => Some("Statement execution is disabled in the settings.".into()),
            None => Some("Statement execution not allowed against database.".into()),
        };

        let actions = parser
            .iter_with_filter(
                DefaultMapper,
                CursorPositionFilter::new(params.cursor_position),
            )
            .map(|(stmt, _, txt)| {
                let title = format!(
                    "Execute Statement: {}...",
                    txt.chars().take(50).collect::<String>()
                );

                CodeAction {
                    title,
                    kind: CodeActionKind::Command(CommandAction {
                        category: CommandActionCategory::ExecuteStatement(stmt),
                    }),
                    disabled_reason: disabled_reason.clone(),
                }
            })
            .collect();

        Ok(CodeActionsResult { actions })
    }

    fn execute_statement(
        &self,
        params: ExecuteStatementParams,
    ) -> Result<ExecuteStatementResult, WorkspaceError> {
        let parser = self
            .parsed_documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let stmt = parser.find(params.statement_id, ExecuteStatementMapper);

        if stmt.is_none() {
            return Ok(ExecuteStatementResult {
                message: "Statement was not found in document.".into(),
            });
        };

        let (_id, _range, content, ast) = stmt.unwrap();

        if ast.is_none() {
            return Ok(ExecuteStatementResult {
                message: "Statement is invalid.".into(),
            });
        };

        let pool = self.get_current_connection();
        if pool.is_none() {
            return Ok(ExecuteStatementResult {
                message: "No database connection available.".into(),
            });
        }
        let pool = pool.unwrap();

        let result = run_async(async move { pool.execute(sqlx::query(&content)).await })??;

        Ok(ExecuteStatementResult {
            message: format!(
                "Successfully executed statement. Rows affected: {}",
                result.rows_affected()
            ),
        })
    }

    fn pull_diagnostics(
        &self,
        params: PullDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        let settings = self.workspaces();

        let settings = match settings.settings() {
            Some(settings) => settings,
            None => {
                // return an empty result if no settings are available
                // we might want to return an error here in the future
                return Ok(PullDiagnosticsResult {
                    diagnostics: Vec::new(),
                    errors: 0,
                    skipped_diagnostics: 0,
                });
            }
        };

        // create analyser for this run
        // first, collect enabled and disabled rules from the workspace settings
        let (enabled_rules, disabled_rules) = AnalyserVisitorBuilder::new(settings)
            .with_linter_rules(&params.only, &params.skip)
            .finish();
        // then, build a map that contains all options
        let options = AnalyserOptions {
            rules: to_analyser_rules(settings),
        };
        // next, build the analysis filter which will be used to match rules
        let filter = AnalysisFilter {
            categories: params.categories,
            enabled_rules: Some(enabled_rules.as_slice()),
            disabled_rules: &disabled_rules,
        };
        // finally, create the analyser that will be used during this run
        let analyser = Analyser::new(AnalyserConfig {
            options: &options,
            filter,
        });

        let parser = self
            .parsed_documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let mut diagnostics: Vec<SDiagnostic> = parser.document_diagnostics().to_vec();

        if let Some(pool) = self.get_current_connection() {
            let path_clone = params.path.clone();
            let schema_cache = self.schema_cache.load(pool.clone())?;
            let input = parser.iter(AsyncDiagnosticsMapper).collect::<Vec<_>>();
            // sorry for the ugly code :(
            let async_results = run_async(async move {
                stream::iter(input)
                    .map(|(_id, range, content, ast, cst, sign)| {
                        let pool = pool.clone();
                        let path = path_clone.clone();
                        let schema_cache = Arc::clone(&schema_cache);
                        async move {
                            if let Some(ast) = ast {
                                pgt_typecheck::check_sql(TypecheckParams {
                                    conn: &pool,
                                    sql: &content,
                                    ast: &ast,
                                    tree: &cst,
                                    schema_cache: schema_cache.as_ref(),
                                    identifiers: sign
                                        .map(|s| {
                                            s.args
                                                .iter()
                                                .map(|a| TypedIdentifier {
                                                    path: s.name.clone(),
                                                    name: a.name.clone(),
                                                    type_: IdentifierType {
                                                        schema: a.type_.schema.clone(),
                                                        name: a.type_.name.clone(),
                                                        is_array: a.type_.is_array,
                                                    },
                                                })
                                                .collect::<Vec<_>>()
                                        })
                                        .unwrap_or_default(),
                                })
                                .await
                                .map(|d| {
                                    d.map(|d| {
                                        let r = d.location().span.map(|span| span + range.start());

                                        d.with_file_path(path.as_path().display().to_string())
                                            .with_file_span(r.unwrap_or(range))
                                    })
                                })
                            } else {
                                Ok(None)
                            }
                        }
                    })
                    .buffer_unordered(10)
                    .collect::<Vec<_>>()
                    .await
            })?;

            for result in async_results.into_iter() {
                let result = result?;
                if let Some(diag) = result {
                    diagnostics.push(SDiagnostic::new(diag));
                }
            }
        }

        diagnostics.extend(parser.iter(SyncDiagnosticsMapper).flat_map(
            |(_id, range, ast, diag)| {
                let mut errors: Vec<Error> = vec![];

                if let Some(diag) = diag {
                    errors.push(diag.into());
                }

                if let Some(ast) = ast {
                    errors.extend(
                        analyser
                            .run(AnalyserContext { root: &ast })
                            .into_iter()
                            .map(Error::from)
                            .collect::<Vec<pgt_diagnostics::Error>>(),
                    );
                }

                errors
                    .into_iter()
                    .map(|d| {
                        let severity = d
                            .category()
                            .filter(|category| category.name().starts_with("lint/"))
                            .map_or_else(
                                || d.severity(),
                                |category| {
                                    settings
                                        .get_severity_from_rule_code(category)
                                        .unwrap_or(Severity::Warning)
                                },
                            );

                        SDiagnostic::new(
                            d.with_file_path(params.path.as_path().display().to_string())
                                .with_file_span(range)
                                .with_severity(severity),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        ));

        let errors = diagnostics
            .iter()
            .filter(|d| d.severity() == Severity::Error || d.severity() == Severity::Fatal)
            .count();

        info!("Pulled {:?} diagnostic(s)", diagnostics.len());
        Ok(PullDiagnosticsResult {
            diagnostics,
            errors,
            skipped_diagnostics: 0,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn get_completions(
        &self,
        params: GetCompletionsParams,
    ) -> Result<CompletionsResult, WorkspaceError> {
        let parsed_doc = self
            .parsed_documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let pool = self.get_current_connection();
        if pool.is_none() {
            tracing::debug!("No database connection available. Skipping completions.");
            return Ok(CompletionsResult::default());
        }
        let pool = pool.unwrap();

        let schema_cache = self.schema_cache.load(pool)?;

        match get_statement_for_completions(&parsed_doc, params.position) {
            None => {
                tracing::debug!("No statement found.");
                Ok(CompletionsResult::default())
            }
            Some((id, range, content, cst)) => {
                let position = params.position - range.start();

                let items = pgt_completions::complete(pgt_completions::CompletionParams {
                    position,
                    schema: schema_cache.as_ref(),
                    tree: &cst,
                    text: content,
                });

                tracing::debug!(
                    "Found {} completion items for statement with id {}",
                    items.len(),
                    id.raw()
                );

                Ok(CompletionsResult { items })
            }
        }
    }
}

/// Returns `true` if `path` is a directory or
/// if it is a symlink that resolves to a directory.
fn is_dir(path: &Path) -> bool {
    path.is_dir() || (path.is_symlink() && fs::read_link(path).is_ok_and(|path| path.is_dir()))
}
