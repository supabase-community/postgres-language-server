use std::{
    collections::HashMap,
    fs,
    panic::RefUnwindSafe,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use analyser::AnalyserVisitorBuilder;
use async_helper::run_async;
use connection_manager::ConnectionManager;
use document::{
    CursorPositionFilter, DefaultMapper, Document, ExecuteStatementMapper,
    TypecheckDiagnosticsMapper,
};
use futures::{StreamExt, stream};
use pg_query::convert_to_positional_params;
use pgls_analyse::{AnalyserOptions, AnalysisFilter};
use pgls_analyser::{Analyser, AnalyserConfig, AnalyserParams};
use pgls_diagnostics::{
    Diagnostic, DiagnosticExt, Error, Severity, serde::Diagnostic as SDiagnostic,
};
use pgls_fs::{ConfigName, PgLSPath};
use pgls_typecheck::{IdentifierType, TypecheckParams, TypedIdentifier};
use pgls_workspace_macros::ignored_path;
use schema_cache_manager::SchemaCacheManager;
use sqlx::{Executor, PgPool};
use tracing::{debug, info};

use crate::{
    WorkspaceError,
    configuration::to_analyser_rules,
    features::{
        code_actions::{
            CodeAction, CodeActionKind, CodeActionsParams, CodeActionsResult, CommandAction,
            CommandActionCategory, ExecuteStatementParams, ExecuteStatementResult,
        },
        completions::{CompletionsResult, GetCompletionsParams, get_statement_for_completions},
        diagnostics::{PullDiagnosticsParams, PullDiagnosticsResult},
        on_hover::{OnHoverParams, OnHoverResult},
    },
    settings::{WorkspaceSettings, WorkspaceSettingsHandle, WorkspaceSettingsHandleMut},
    workspace::{AnalyserDiagnosticsMapper, WithCSTandASTMapper},
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
mod connection_key;
mod connection_manager;
pub(crate) mod document;
mod migration;
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

    documents: RwLock<HashMap<PgLSPath, Document>>,

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
            documents: RwLock::new(HashMap::new()),
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
    fn get_current_project_path(&self) -> Option<PgLSPath> {
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
    fn path_belongs_to_current_workspace(&self, path: &PgLSPath) -> Option<ProjectKey> {
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
        // Never ignore config files regardless `include`/`ignore`
        if self.is_config_file(path) {
            return false;
        }

        // Apply top-level `include`/`ignore`
        self.is_ignored_by_top_level_config(path) || self.is_ignored_by_migration_config(path)
    }

    /// Check whether a file is a configuration file
    fn is_config_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|file_name| ConfigName::file_names().contains(&file_name))
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
    #[ignored_path(path=&params.path)]
    #[tracing::instrument(level = "info", skip_all, fields(path = params.path.as_path().as_os_str().to_str()), err)]
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write().unwrap();
        documents
            .entry(params.path.clone())
            .or_insert_with(|| Document::new(params.content, params.version));

        if let Some(project_key) = self.path_belongs_to_current_workspace(&params.path) {
            self.set_current_project(project_key);
        }

        Ok(())
    }

    /// Remove a file from the workspace
    #[ignored_path(path=&params.path)]
    fn close_file(&self, params: super::CloseFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write().unwrap();
        documents
            .remove(&params.path)
            .ok_or_else(WorkspaceError::not_found)?;

        Ok(())
    }

    /// Change the content of an open file
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        version = params.version
    ), err)]
    #[ignored_path(path=&params.path)]
    fn change_file(&self, params: super::ChangeFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write().unwrap();

        match documents.entry(params.path.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .update_content(params.content, params.version);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Document::new(params.content, params.version));
            }
        }

        Ok(())
    }

    fn server_info(&self) -> Option<&ServerInfo> {
        None
    }

    #[ignored_path(path=&params.path)]
    fn get_file_content(&self, params: GetFileContentParams) -> Result<String, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let document = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;
        Ok(document.get_document_content().to_string())
    }

    fn is_path_ignored(&self, params: IsPathIgnoredParams) -> Result<bool, WorkspaceError> {
        Ok(self.is_ignored(params.pgls_path.as_path()))
    }

    #[ignored_path(path=&params.path)]
    fn pull_code_actions(
        &self,
        params: CodeActionsParams,
    ) -> Result<CodeActionsResult, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let parser = documents
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

    #[ignored_path(path=&params.path)]
    fn execute_statement(
        &self,
        params: ExecuteStatementParams,
    ) -> Result<ExecuteStatementResult, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let parser = documents
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

        let Some(pool) = self.get_current_connection() else {
            return Ok(ExecuteStatementResult {
                message: "No database connection available.".into(),
            });
        };

        let result = run_async(async move { pool.execute(sqlx::query(&content)).await })??;

        Ok(ExecuteStatementResult {
            message: format!(
                "Successfully executed statement. Rows affected: {}",
                result.rows_affected()
            ),
        })
    }

    #[ignored_path(path=&params.path)]
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

        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        /*
         * The statements in the document might already have associated diagnostics,
         * e.g. if they contain syntax errors that surfaced while parsing/splitting the statements
         */
        let mut diagnostics: Vec<SDiagnostic> = doc.document_diagnostics().to_vec();

        /*
         * Type-checking against database connection
         */
        let typecheck_enabled = settings.typecheck.enabled;
        let plpgsql_check_enabled = settings.plpgsql_check.enabled;
        if typecheck_enabled || plpgsql_check_enabled {
            if let Some(pool) = self.get_current_connection() {
                let path_clone = params.path.clone();
                let schema_cache = self.schema_cache.load(pool.clone())?;
                let input = doc.iter(TypecheckDiagnosticsMapper).collect::<Vec<_>>();
                let search_path_patterns = settings.typecheck.search_path.clone();

                // Combined async context for both typecheck and plpgsql_check
                let async_results = run_async(async move {
                    stream::iter(input)
                        .map(|(id, range, ast, cst, fn_sig)| {
                            let pool = pool.clone();
                            let path = path_clone.clone();
                            let schema_cache = Arc::clone(&schema_cache);
                            let search_path_patterns = search_path_patterns.clone();

                            async move {
                                let mut diagnostics = Vec::new();

                                if let Some(ast) = ast {
                                    // Type checking
                                    if typecheck_enabled {
                                        let typecheck_result =
                                            pgls_typecheck::check_sql(TypecheckParams {
                                                conn: &pool,
                                                sql: convert_to_positional_params(id.content())
                                                    .as_str(),
                                                ast: &ast,
                                                tree: &cst,
                                                schema_cache: schema_cache.as_ref(),
                                                search_path_patterns,
                                                identifiers: fn_sig
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
                                            .await;

                                        if let Ok(Some(diag)) = typecheck_result {
                                            let r = diag
                                                .location()
                                                .span
                                                .map(|span| span + range.start());
                                            diagnostics.push(
                                                diag.with_file_path(
                                                    path.as_path().display().to_string(),
                                                )
                                                .with_file_span(r.unwrap_or(range)),
                                            );
                                        }
                                    }

                                    // plpgsql_check
                                    if plpgsql_check_enabled {
                                        let plpgsql_check_results =
                                            pgls_plpgsql_check::check_plpgsql(
                                                pgls_plpgsql_check::PlPgSqlCheckParams {
                                                    conn: &pool,
                                                    sql: id.content(),
                                                    ast: &ast,
                                                    schema_cache: schema_cache.as_ref(),
                                                },
                                            )
                                            .await
                                            .unwrap_or_else(|_| vec![]);

                                        for d in plpgsql_check_results {
                                            let r = d.span.map(|span| span + range.start());
                                            diagnostics.push(
                                                d.with_file_path(
                                                    path.as_path().display().to_string(),
                                                )
                                                .with_file_span(r.unwrap_or(range)),
                                            );
                                        }
                                    }
                                }

                                Ok::<Vec<pgls_diagnostics::Error>, sqlx::Error>(diagnostics)
                            }
                        })
                        .buffer_unordered(10)
                        .collect::<Vec<_>>()
                        .await
                })?;

                for result in async_results.into_iter() {
                    let diagnostics_batch = result?;
                    for diag in diagnostics_batch {
                        diagnostics.push(SDiagnostic::new(diag));
                    }
                }
            }
        }

        /*
         * Below, we'll apply our static linting rules against the statements,
         * considering the user's settings
         */
        let (enabled_rules, disabled_rules) = AnalyserVisitorBuilder::new(settings)
            .with_linter_rules(&params.only, &params.skip)
            .finish();

        let options = AnalyserOptions {
            rules: to_analyser_rules(settings),
        };

        let filter = AnalysisFilter {
            categories: params.categories,
            enabled_rules: Some(enabled_rules.as_slice()),
            disabled_rules: &disabled_rules,
        };

        let analyser = Analyser::new(AnalyserConfig {
            options: &options,
            filter,
        });

        let path = params.path.as_path().display().to_string();

        let schema_cache = self
            .get_current_connection()
            .and_then(|pool| self.schema_cache.load(pool.clone()).ok());

        let mut analysable_stmts = vec![];
        for (stmt_root, diagnostic) in doc.iter(AnalyserDiagnosticsMapper) {
            if let Some(node) = stmt_root {
                analysable_stmts.push(node);
            }
            if let Some(diag) = diagnostic {
                // ignore the syntax error if we already have more specialized diagnostics for the
                // same statement.
                // this is important for create function statements, where we might already have detailed
                // diagnostics from plpgsql_check.
                if diagnostics.iter().any(|d| {
                    d.location().span.is_some_and(|async_loc| {
                        diag.location()
                            .span
                            .is_some_and(|syntax_loc| syntax_loc.contains_range(async_loc))
                    })
                }) {
                    continue;
                }

                diagnostics.push(SDiagnostic::new(
                    diag.with_file_path(path.clone())
                        .with_severity(Severity::Error),
                ));
            }
        }

        diagnostics.extend(
            analyser
                .run(AnalyserParams {
                    stmts: analysable_stmts,
                    schema_cache: schema_cache.as_deref(),
                })
                .into_iter()
                .map(Error::from)
                .map(|d| {
                    let severity = d
                        .category()
                        .map(|category| {
                            settings
                                .get_severity_from_rule_code(category)
                                .unwrap_or(Severity::Warning)
                        })
                        .unwrap();

                    let span = d.location().span;
                    SDiagnostic::new(
                        d.with_file_path(path.clone())
                            .with_file_span(span)
                            .with_severity(severity),
                    )
                }),
        );

        let suppressions = doc.suppressions();

        let disabled_suppression_errors =
            suppressions.get_disabled_diagnostic_suppressions_as_errors(&disabled_rules);

        let unused_suppression_errors =
            suppressions.get_unused_suppressions_as_errors(&diagnostics);

        let suppression_errors: Vec<Error> = suppressions
            .diagnostics
            .iter()
            .chain(disabled_suppression_errors.iter())
            .chain(unused_suppression_errors.iter())
            .cloned()
            .map(Error::from)
            .collect::<Vec<pgls_diagnostics::Error>>();

        diagnostics.retain(|d| !suppressions.is_suppressed(d));
        diagnostics.extend(suppression_errors.into_iter().map(SDiagnostic::new));

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

    #[ignored_path(path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn get_completions(
        &self,
        params: GetCompletionsParams,
    ) -> Result<CompletionsResult, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let parsed_doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let Some(pool) = self.get_current_connection() else {
            tracing::debug!("No database connection available. Skipping completions.");
            return Ok(CompletionsResult::default());
        };

        let schema_cache = self.schema_cache.load(pool.clone())?;

        match get_statement_for_completions(parsed_doc, params.position) {
            None => {
                tracing::debug!("No statement found.");
                Ok(CompletionsResult::default())
            }
            Some((id, range, cst)) => {
                let position = params.position - range.start();

                let items = pgls_completions::complete(pgls_completions::CompletionParams {
                    position,
                    schema: schema_cache.as_ref(),
                    tree: &cst,
                    text: id.content().to_string(),
                });

                Ok(CompletionsResult { items })
            }
        }
    }

    #[ignored_path(path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn on_hover(&self, params: OnHoverParams) -> Result<OnHoverResult, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let Some(pool) = self.get_current_connection() else {
            tracing::debug!("No database connection available. Skipping completions.");
            return Ok(OnHoverResult::default());
        };

        let schema_cache = self.schema_cache.load(pool.clone())?;

        match doc
            .iter_with_filter(
                WithCSTandASTMapper,
                CursorPositionFilter::new(params.position),
            )
            .next()
        {
            Some((stmt_id, range, ts_tree, maybe_ast)) => {
                let position_in_stmt = params.position - range.start();

                let markdown_blocks = pgls_hover::on_hover(pgls_hover::OnHoverParams {
                    ts_tree: &ts_tree,
                    schema_cache: &schema_cache,
                    ast: maybe_ast.as_ref(),
                    position: position_in_stmt,
                    stmt_sql: stmt_id.content(),
                });

                Ok(OnHoverResult { markdown_blocks })
            }
            None => Ok(OnHoverResult::default()),
        }
    }
}

/// Returns `true` if `path` is a directory or
/// if it is a symlink that resolves to a directory.
fn is_dir(path: &Path) -> bool {
    path.is_dir() || (path.is_symlink() && fs::read_link(path).is_ok_and(|path| path.is_dir()))
}

#[cfg(test)]
#[path = "server.tests.rs"]
mod tests;
