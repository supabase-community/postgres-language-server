use std::{
    collections::HashMap,
    panic::RefUnwindSafe,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use analyser::AnalyserVisitorBuilder;
use analysis::AnalysisRequest;
#[cfg(feature = "db")]
use analysis::DocumentAnalysis;
#[cfg(feature = "db")]
use async_helper::run_async;
#[cfg(feature = "db")]
use connection_manager::ConnectionManager;
#[cfg(feature = "db")]
use document::TypecheckDiagnosticsMapper;
use document::{DefaultMapper, Document, ExecuteStatementMapper};
use document_cache::{CacheBudget, CacheKey};
pub use document_cache::{DOCUMENT_CACHE_ENTRIES_MAX, DocumentCacheStats};
#[cfg(feature = "db")]
use futures::{StreamExt, TryStreamExt, stream};
use parking_lot::{Mutex, RwLock};
use pgls_analyse::{ActionCategory, AnalysisFilter, Applicability, SUPPRESSION_ACTION_CATEGORY};
use pgls_analyser::{LinterDiagnostic, MigrationSection, RuleFix};
use pgls_catalog::CatalogBase;
#[cfg(feature = "db")]
use pgls_lexer::convert_to_positional_params_with_metadata;
use std::borrow::Cow;

use pgls_catalog::Snapshot;
use pgls_diagnostics::{
    Diagnostic, DiagnosticExt, Error, Severity, serde::Diagnostic as SDiagnostic,
};
use pgls_fs::{ConfigName, PgLSPath};
use pgls_text_size::{TextRange, TextSize};
#[cfg(feature = "db")]
use pgls_typecheck::TypecheckParams;
use pgls_workspace_macros::ignored_path;
use snapshot_manager::SnapshotManager;
use tracing::{debug, info, warn};

use crate::{
    WorkspaceError,
    features::{
        code_actions::{
            CodeAction, CodeActionKind, CodeActionsParams, CodeActionsResult, CommandAction,
            CommandActionCategory, EditAction, ExecuteStatementParams, ExecuteStatementResult,
            FixFileMode, FixFileParams, FixFileResult, TextEdit,
        },
        completions::{CompletionsResult, GetCompletionsParams, get_statement_for_completions},
        diagnostics::{
            CheckDatabaseConnectionParams, CheckDatabaseConnectionResult, PullDiagnosticsResult,
            PullFileDiagnosticsParams,
        },
        format::{PullFileFormattingParams, PullFormattingResult},
        inlay_hints::{InlayHint, PullInlayHintsParams, PullInlayHintsResult},
        on_hover::{OnHoverParams, OnHoverResult},
        semantic_tokens::{PullSemanticTokensParams, PullSemanticTokensResult, SemanticToken},
    },
    settings::{Settings, WorkspaceSettings, WorkspaceSettingsHandle, WorkspaceSettingsHandleMut},
    workspace::WithCSTMapper,
};

use super::{
    GetFileContentParams, IsPathIgnoredParams, NestedSettingsError, OpenFileParams, ProjectKey,
    RegisterProjectFolderParams, ServerInfo, UnregisterProjectFolderParams,
    UpdateNestedSettingsParams, UpdateNestedSettingsResult, UpdateSettingsParams, Workspace,
};

pub use statement_identifier::StatementId;

mod analyser;
mod analysis;
#[cfg(feature = "db")]
mod async_helper;
#[cfg(feature = "db")]
mod connection_key;
#[cfg(feature = "db")]
mod connection_manager;
pub(crate) mod document;
mod document_cache;
mod expand_star;
mod formatting;
mod migration;
#[cfg(feature = "db")]
mod pgpass;
mod snapshot_manager;
mod sql_function;
#[cfg(feature = "db")]
mod statement_execution;
mod statement_identifier;
mod walk;

pub struct WorkspaceServer {
    /// global settings object for this workspace
    settings: RwLock<WorkspaceSettings>,

    /// The open documents. Requests work on an `Arc` of a document, so that an edit doesn't wait
    /// for them; it copies the document if a request still uses it.
    documents: RwLock<HashMap<PgLSPath, Arc<Document>>>,

    /// The database snapshots, loaded from a database connection or from JSON
    snapshot: SnapshotManager,

    /// The indexed database snapshot the catalog starts from
    catalog_base: Mutex<Option<Arc<CatalogBase>>>,

    /// Incremented whenever the settings change, see [`CacheKey`].
    settings_version: AtomicU64,

    /// What the caches of the open documents keep together, see
    /// [`DOCUMENT_CACHE_ENTRIES_MAX`].
    document_caches: CacheBudget<PgLSPath>,

    #[cfg(feature = "db")]
    connection: ConnectionManager,

    /// The statements that the editor walks visited, to test that they resume from
    /// checkpoints.
    #[cfg(test)]
    walked_statements: std::sync::atomic::AtomicUsize,
}

/// The `Workspace` object is long-lived, so we want it to be able to cross
/// unwind boundaries.
/// In return, we have to make sure operations on the workspace either do not
/// panic, of that panicking will not result in any broken invariant (it would
/// not result in any undefined behavior as catching an unwind is safe, but it
/// could lead too hard to debug issues)
impl RefUnwindSafe for WorkspaceServer {}

impl Default for WorkspaceServer {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceServer {
    /// Create a new [Workspace]
    #[cfg(feature = "db")]
    pub fn new() -> Self {
        Self {
            settings: RwLock::default(),
            documents: RwLock::new(HashMap::new()),
            snapshot: SnapshotManager::new(),
            catalog_base: Mutex::default(),
            settings_version: AtomicU64::default(),
            document_caches: CacheBudget::new(DOCUMENT_CACHE_ENTRIES_MAX),
            connection: ConnectionManager::new(),
            #[cfg(test)]
            walked_statements: Default::default(),
        }
    }

    /// Create a new [Workspace] without database support
    #[cfg(not(feature = "db"))]
    pub fn new() -> Self {
        Self {
            settings: RwLock::default(),
            documents: RwLock::new(HashMap::new()),
            snapshot: SnapshotManager::new(),
            catalog_base: Mutex::default(),
            settings_version: AtomicU64::default(),
            document_caches: CacheBudget::new(DOCUMENT_CACHE_ENTRIES_MAX),
            #[cfg(test)]
            walked_statements: Default::default(),
        }
    }

    /// Provides a reference to the current settings
    fn workspaces(&self) -> WorkspaceSettingsHandle<'_> {
        WorkspaceSettingsHandle::new(&self.settings)
    }

    fn workspaces_mut(&self) -> WorkspaceSettingsHandleMut<'_> {
        let handle = WorkspaceSettingsHandleMut::new(&self.settings);
        // While the settings are locked, so that nothing is cached with the old settings under
        // the new version.
        self.settings_version.fetch_add(1, Ordering::Relaxed);
        handle
    }

    /// What the caches of the open document at `path` hold, for tests of their memory.
    #[doc(hidden)]
    pub fn document_cache_stats(&self, path: &PgLSPath) -> Option<DocumentCacheStats> {
        let documents = self.documents.read();
        documents.get(path).map(|doc| doc.cache_stats())
    }

    /// The open document at `path`. The lock of the documents is only held to look it up.
    fn document(&self, path: &PgLSPath) -> Result<Arc<Document>, WorkspaceError> {
        let documents = self.documents.read();
        documents
            .get(path)
            .cloned()
            .ok_or_else(WorkspaceError::not_found)
    }

    /// Whether a request on `doc` should stop: the client cancelled it, or changed the document
    /// at `path` while it was running, so its result would be outdated.
    pub(super) fn should_stop<'a>(
        &'a self,
        path: &'a PgLSPath,
        doc: &'a Document,
    ) -> impl Fn() -> bool + 'a {
        move || crate::cancellation::is_cancelled() || self.is_outdated(path, doc)
    }

    /// Whether `doc` is no longer the content of the document at `path`, e.g. because the client
    /// changed it while a request was running.
    pub(super) fn is_outdated(&self, path: &PgLSPath, doc: &Document) -> bool {
        let documents = self.documents.read();
        documents
            .get(path)
            .is_none_or(|current| current.generation() != doc.generation())
    }

    /// Records what the caches of the document at `path` keep after a request, and drops the
    /// caches of the least recently used other documents to stay within
    /// [`DOCUMENT_CACHE_ENTRIES_MAX`]. The document of the request is never dropped, so a
    /// single file larger than the budget stays cached while it is used.
    fn charge_document_caches(&self, path: &PgLSPath, doc: &Document) {
        let evicted = self.document_caches.record(path, doc.cache().usage());
        if evicted.is_empty() {
            return;
        }
        let documents = self.documents.read();
        for evicted in evicted {
            if let Some(doc) = documents.get(&evicted) {
                tracing::debug!(
                    "Dropping the caches of {} to keep at most {} cache entries",
                    evicted.display(),
                    self.document_caches.max()
                );
                doc.cache().clear();
            }
        }
    }

    /// The database snapshot of the database in `settings`, if it is reachable or a snapshot was
    /// loaded from JSON.
    fn database_snapshot(&self, settings: &Settings) -> Option<Arc<Snapshot>> {
        #[cfg(feature = "db")]
        let snapshot = self
            .connection
            .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            .and_then(Result::ok);

        #[cfg(not(feature = "db"))]
        let snapshot = {
            let _ = settings;
            self.snapshot.get()
        };

        snapshot
    }

    /// What the caches of a document of `project_key` depend on besides its text, when the
    /// analysis starts from `snapshot`.
    fn cache_key(&self, project_key: ProjectKey, snapshot: Option<&Arc<Snapshot>>) -> CacheKey {
        CacheKey::new(
            self.settings_version.load(Ordering::Relaxed),
            project_key,
            snapshot.map(|snapshot| self.catalog_base(snapshot)),
        )
    }

    /// Load schema from JSON string.
    /// This allows setting a database snapshot without a database connection.
    pub fn set_schema_json(&self, json: &str) -> Result<(), WorkspaceError> {
        self.snapshot.set(json)
    }

    /// Clear the schema.
    pub fn clear_schema(&self) {
        self.snapshot.clear();
    }

    /// Split raw SQL into byte ranges of individual statements.
    #[allow(clippy::unused_self)]
    pub fn split_statements(&self, sql: &str) -> Vec<TextRange> {
        pgls_statement_splitter::split(sql).ranges
    }

    /// Get a clone of the current schema.
    pub fn get_schema(&self) -> Option<Arc<Snapshot>> {
        self.snapshot.get()
    }

    /// Register a new project in the current workspace
    fn register_project(&self, path: PathBuf) -> ProjectKey {
        let mut workspace = self.workspaces_mut();
        let workspace_mut = workspace.as_mut();
        workspace_mut.insert_project(path.clone())
    }

    /// Check whether a file is ignored by the settings of its project that apply to it
    fn is_ignored(&self, project_key: ProjectKey, path: &Path) -> bool {
        // Never ignore config files regardless `include`/`ignore`
        if self.is_config_file(path) {
            return false;
        }

        let settings = self.workspaces();
        let Some(settings) = settings.settings_for(project_key, path) else {
            return false;
        };

        // Apply top-level `include`/`ignore`
        settings.files.is_ignored(path) || is_ignored_by_migration_settings(settings, path)
    }

    /// The fixes of the lint diagnostics at the cursor, and an action to suppress each of them.
    fn lint_code_actions(
        &self,
        settings: &Settings,
        params: &CodeActionsParams,
        doc: &Document,
    ) -> Vec<CodeAction> {
        // Editors request code actions whenever the cursor moves. Without a rule selection,
        // this is the analysis that the diagnostics of the last edit cached.
        let should_stop = self.should_stop(&params.path, doc);
        let Some(analysis) = self.analyse_document(
            params.project_key,
            settings,
            &params.path,
            doc,
            AnalysisRequest::all(&params.only, &params.skip, &should_stop),
        ) else {
            return Vec::new();
        };
        let suppressions = doc.suppressions();
        let mut actions = Vec::new();
        for diagnostic in analysis.lint_diagnostics() {
            let Some(span) = diagnostic.location().span else {
                continue;
            };
            if !span.contains_inclusive(params.cursor_position)
                || suppressions.is_suppressed(diagnostic)
            {
                continue;
            }
            let category = diagnostic.get_category_name();
            let rule = category.strip_prefix("lint/").unwrap_or(category);

            if let Some(fix) = diagnostic.get_fix() {
                actions.push(CodeAction {
                    title: fix.message.clone(),
                    kind: CodeActionKind::Edit(EditAction {
                        rule: Some(category.to_string()),
                        category: ActionCategory::QuickFix(Cow::Owned(rule.to_string())),
                        applicability: fix.applicability,
                        edits: fix
                            .edits
                            .iter()
                            .map(|edit| TextEdit {
                                range: edit.range,
                                new_text: edit.replacement.clone(),
                            })
                            .collect(),
                    }),
                    disabled_reason: None,
                });
            }

            actions.push(CodeAction {
                title: format!("Suppress rule {rule} for this statement"),
                kind: CodeActionKind::Edit(EditAction {
                    rule: Some(category.to_string()),
                    category: ActionCategory::Other(Cow::Borrowed(SUPPRESSION_ACTION_CATEGORY)),
                    applicability: Applicability::MaybeIncorrect,
                    edits: vec![suppression_edit(doc, span.start(), rule)],
                }),
                disabled_reason: None,
            });
        }
        actions
    }

    /// The refactors offered at `position`: expanding a `*` into its columns. `settings` are the
    /// ones of the file.
    fn refactor_code_actions(
        &self,
        project_key: ProjectKey,
        path: &PgLSPath,
        settings: Option<&Settings>,
        doc: &Document,
        position: TextSize,
    ) -> Vec<CodeAction> {
        // Code actions are requested whenever the cursor moves, so don't resolve the file unless
        // the cursor is on a `*` of a select list.
        let content = doc.get_document_content();
        if !expand_star::is_at_star(content, position) {
            return Vec::new();
        }

        #[cfg(feature = "db")]
        let snapshot = settings.and_then(|settings| {
            self.connection
                .with_pool(&settings.db, |pool| self.snapshot.load(pool))
                .and_then(Result::ok)
        });

        #[cfg(not(feature = "db"))]
        let snapshot = self.snapshot.get();

        let Some(snapshot) = snapshot else {
            return Vec::new();
        };
        let request = self.editor_request(project_key, path, settings, snapshot);
        let (stmt_start, references) = self.references_at(&request, doc, position);
        references
            .iter()
            .filter_map(|reference| expand_star::expand_star_action(reference, content, stmt_start))
            .collect()
    }

    /// Diagnostics from the database: EXPLAIN for top-level statements whose references are all
    /// unchanged database objects, and plpgsql_check. Also returns the ranges of the
    /// statements that plpgsql_check checked.
    #[cfg(feature = "db")]
    fn database_diagnostics(
        &self,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        analysis: &DocumentAnalysis,
    ) -> Result<(Vec<SDiagnostic>, Vec<TextRange>), WorkspaceError> {
        let typecheck_enabled = settings.typecheck.enabled;
        let plpgsql_check = settings.plpgsql_check.clone();
        if !typecheck_enabled && !plpgsql_check.enabled {
            return Ok((Vec::new(), Vec::new()));
        }

        let db_diagnostics = self.connection.with_pool(&settings.db, |pool| {
            let snapshot = self.snapshot.load(pool)?;
            let input = doc
                .iter(TypecheckDiagnosticsMapper)
                .filter(|(_, range, ..)| !analysis.is_skipped(range.start()))
                .map(|(id, range, ast, cst)| {
                    let statement = analysis
                        .statements()
                        .iter()
                        .find(|statement| statement.range == range)
                        .cloned();
                    (id, range, ast, cst, statement)
                })
                .collect::<Vec<_>>();
            let pool = pool.clone();
            let path = path.clone();

            run_async(async move {
                stream::iter(input)
                    .map(|(id, range, ast, cst, statement)| {
                        let pool = pool.clone();
                        let path = path.clone();
                        let snapshot = Arc::clone(&snapshot);
                        let plpgsql_check = plpgsql_check.clone();

                        async move {
                            let mut diagnostics = Vec::new();
                            let mut plpgsql_checked = None;
                            let Some(ast) = ast else {
                                return Ok((diagnostics, plpgsql_checked));
                            };

                            // The static typecheck rules cover everything else, including the
                            // statements of SQL function bodies, whose parameters they resolve.
                            let statement = statement.filter(|statement| {
                                typecheck_enabled && statement.database_only && !id.is_child()
                            });
                            if let Some(statement) = statement {
                                let conversion =
                                    convert_to_positional_params_with_metadata(id.content());

                                if !conversion.has_identifier_parameters {
                                    let result = pgls_typecheck::check_sql(TypecheckParams {
                                        conn: &pool,
                                        sql: conversion.sql.as_str(),
                                        ast: &ast,
                                        tree: &cst,
                                        snapshot: snapshot.as_ref(),
                                        search_path: &statement.search_path,
                                        identifiers: Vec::new(),
                                    })
                                    .await?;

                                    if let Some(diagnostic) = result {
                                        let span = diagnostic
                                            .location()
                                            .span
                                            .map(|span| span + range.start());
                                        diagnostics.push(
                                            diagnostic
                                                .with_file_path(
                                                    path.as_path().display().to_string(),
                                                )
                                                .with_file_span(span.unwrap_or(range)),
                                        );
                                    }
                                }
                            }

                            if plpgsql_check.enabled {
                                let results = pgls_plpgsql_check::check_plpgsql(
                                    pgls_plpgsql_check::PlPgSqlCheckParams {
                                        conn: &pool,
                                        sql: id.content(),
                                        ast: &ast,
                                        snapshot: snapshot.as_ref(),
                                        fatal_errors: plpgsql_check.fatal_errors,
                                        other_warnings: plpgsql_check.other_warnings,
                                        extra_warnings: plpgsql_check.extra_warnings,
                                        performance_warnings: plpgsql_check.performance_warnings,
                                        security_warnings: plpgsql_check.security_warnings,
                                        compatibility_warnings: plpgsql_check
                                            .compatibility_warnings,
                                        without_warnings: plpgsql_check.without_warnings,
                                        all_warnings: plpgsql_check.all_warnings,
                                        use_incomment_options: plpgsql_check.use_incomment_options,
                                        incomment_options_usage_warning: plpgsql_check
                                            .incomment_options_usage_warning,
                                        constant_tracing: plpgsql_check.constant_tracing,
                                    },
                                )
                                .await
                                .ok()
                                .flatten();

                                if results.is_some() {
                                    plpgsql_checked = Some(range);
                                }

                                for diagnostic in results.unwrap_or_default() {
                                    let span = diagnostic.span.map(|span| span + range.start());
                                    diagnostics.push(
                                        diagnostic
                                            .with_file_path(path.as_path().display().to_string())
                                            .with_file_span(span.unwrap_or(range)),
                                    );
                                }
                            }

                            Ok::<_, sqlx::Error>((diagnostics, plpgsql_checked))
                        }
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await
            })?
            .map_err(WorkspaceError::from)
        });

        match db_diagnostics {
            None => Ok((Vec::new(), Vec::new())),
            Some(Ok(batches)) => {
                let mut diagnostics = Vec::new();
                let mut plpgsql_checked = Vec::new();
                for (batch, checked) in batches {
                    diagnostics.extend(batch.into_iter().map(SDiagnostic::new));
                    plpgsql_checked.extend(checked);
                }
                Ok((diagnostics, plpgsql_checked))
            }
            Some(Err(err @ WorkspaceError::DatabaseConnectionError(_))) => {
                // Database-backed diagnostics are best-effort. If the connection is
                // unreachable, keep publishing parser/static diagnostics instead of
                // failing the whole edit-triggered diagnostics request.
                debug!("Skipping database-backed diagnostics: {err}");
                Ok((Vec::new(), Vec::new()))
            }
            Some(Err(err)) => Err(err),
        }
    }

    /// Check whether a file is a configuration file
    fn is_config_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|file_name| ConfigName::file_names().contains(&file_name))
    }
}

/// Check whether a file is a migration before `migrations.after`
fn is_ignored_by_migration_settings(settings: &Settings, path: &Path) -> bool {
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

impl Workspace for WorkspaceServer {
    fn register_project_folder(
        &self,
        params: RegisterProjectFolderParams,
    ) -> Result<ProjectKey, WorkspaceError> {
        let path = params.path.unwrap_or_default();
        let existing = self.workspaces().as_ref().get_project_key(&path);
        debug!("Register the project {:?}, known as {:?}", path, existing);

        Ok(existing.unwrap_or_else(|| self.register_project(path)))
    }

    fn unregister_project_folder(
        &self,
        params: UnregisterProjectFolderParams,
    ) -> Result<(), WorkspaceError> {
        let mut workspace = self.workspaces_mut();
        workspace.as_mut().remove_project(params.path.as_path());
        Ok(())
    }

    /// Update the root settings of a project
    ///
    /// ## Panics
    /// This function may panic if the internal settings mutex has been poisoned
    /// by another thread having previously panicked while holding the lock
    #[tracing::instrument(level = "trace", skip(self), err)]
    fn update_settings(&self, params: UpdateSettingsParams) -> Result<(), WorkspaceError> {
        let mut workspace = self.workspaces_mut();

        workspace
            .as_mut()
            .get_settings_mut(params.project_key)
            .ok_or_else(WorkspaceError::no_project)?
            .merge_with_configuration(
                params.configuration,
                params.workspace_directory,
                params.vcs_base_path,
                params.gitignore_matches.as_slice(),
            )?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self), err)]
    fn update_nested_settings(
        &self,
        params: UpdateNestedSettingsParams,
    ) -> Result<UpdateNestedSettingsResult, WorkspaceError> {
        let mut entries = params.nested;
        // The settings above a folder decide whether it is ignored, so start with the shallowest.
        entries.sort_by(|a, b| {
            (a.directory.components().count(), &a.directory)
                .cmp(&(b.directory.components().count(), &b.directory))
        });

        let mut result = UpdateNestedSettingsResult::default();
        let mut nested: Vec<(PathBuf, Settings)> = Vec::new();
        {
            let workspace = self.workspaces();
            let workspace = workspace.as_ref();
            let project_path = workspace
                .get_project_path(params.project_key)
                .ok_or_else(WorkspaceError::no_project)?;
            let root = workspace
                .get_settings(params.project_key)
                .ok_or_else(WorkspaceError::no_project)?;

            for entry in entries {
                let directory = entry.directory;
                if directory == project_path.as_path()
                    || !directory.starts_with(project_path.as_path())
                    || nested.iter().any(|(applied, _)| applied == &directory)
                {
                    continue;
                }
                let above = nested
                    .iter()
                    .rev()
                    .find(|(applied, _)| directory.starts_with(applied))
                    .map_or(root, |(_, settings)| settings);
                if above.files.is_ignored(&directory) {
                    continue;
                }

                // A configuration that cannot be applied falls back to the defaults rather than to
                // the settings above it, which could point at another database.
                let mut settings = Settings::default();
                if let Err(error) = settings.merge_with_configuration(
                    entry.configuration,
                    Some(directory.clone()),
                    entry.vcs_base_path,
                    entry.gitignore_matches.as_slice(),
                ) {
                    result.errors.push(NestedSettingsError {
                        directory: directory.clone(),
                        message: error.to_string(),
                    });
                    settings = Settings::default();
                }
                result.applied.push(directory.clone());
                nested.push((directory, settings));
            }
        }

        self.workspaces_mut()
            .as_mut()
            .set_nested_settings(params.project_key, nested);
        Ok(result)
    }

    /// Add a new file to the workspace
    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "info", skip_all, fields(path = params.path.as_path().as_os_str().to_str()), err)]
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write();
        documents
            .entry(params.path.clone())
            .or_insert_with(|| Arc::new(Document::new(params.content, params.version)));

        Ok(())
    }

    /// Remove a file from the workspace
    #[ignored_path(project=params.project_key, path=&params.path)]
    fn close_file(&self, params: super::CloseFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write();
        documents
            .remove(&params.path)
            .ok_or_else(WorkspaceError::not_found)?;
        self.document_caches.remove(&params.path);

        Ok(())
    }

    /// Change the content of an open file
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        version = params.version
    ), err)]
    #[ignored_path(project=params.project_key, path=&params.path)]
    fn change_file(&self, params: super::ChangeFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write();

        let stale = match documents.entry(params.path.clone()) {
            // A request that still works on the previous version keeps it. The copy shares the
            // document's cache.
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                Arc::make_mut(entry.get_mut()).update_content(params.content, params.version)
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Arc::new(Document::new(params.content, params.version)));
                return Ok(());
            }
        };
        drop(documents);
        stale.drop_in_background();

        Ok(())
    }

    fn server_info(&self) -> Option<&ServerInfo> {
        None
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn get_file_content(&self, params: GetFileContentParams) -> Result<String, WorkspaceError> {
        let document = self.document(&params.path)?;
        Ok(document.get_document_content().to_string())
    }

    fn is_path_ignored(&self, params: IsPathIgnoredParams) -> Result<bool, WorkspaceError> {
        Ok(self.is_ignored(params.project_key, params.pgls_path.as_path()))
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn pull_code_actions(
        &self,
        params: CodeActionsParams,
    ) -> Result<CodeActionsResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());

        let disabled_reason = match settings {
            Some(settings) => settings.db.statement_execution_disabled_reason(),
            None => Some("Statement execution not allowed against database.".into()),
        };

        let mut actions: Vec<CodeAction> = doc
            .iter_at(DefaultMapper, params.cursor_position)
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

        actions.splice(
            0..0,
            self.refactor_code_actions(
                params.project_key,
                &params.path,
                settings,
                doc,
                params.cursor_position,
            ),
        );

        if let Some(settings) = settings {
            let mut lint_actions = self.lint_code_actions(settings, &params, doc);
            lint_actions.append(&mut actions);
            actions = lint_actions;
        }

        #[cfg(feature = "db")]
        let invalidate_disabled_reason = match settings {
            Some(settings)
                if self
                    .connection
                    .with_pool(&settings.db, |_| Ok::<(), WorkspaceError>(()))
                    .is_some() =>
            {
                None
            }
            _ => Some("No database connection available.".into()),
        };

        #[cfg(not(feature = "db"))]
        let invalidate_disabled_reason: Option<String> =
            Some("Database support not enabled.".into());

        actions.push(CodeAction {
            title: "Invalidate Schema Cache".into(),
            kind: CodeActionKind::Command(CommandAction {
                category: CommandActionCategory::InvalidateSchemaCache,
            }),
            disabled_reason: invalidate_disabled_reason,
        });

        self.charge_document_caches(&params.path, doc);
        Ok(CodeActionsResult { actions })
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn fix_file(&self, params: FixFileParams) -> Result<FixFileResult, WorkspaceError> {
        /// Fixes can enable other fixes, e.g. `CONCURRENTLY` enables `IF NOT EXISTS`, and fixes
        /// that overlap are applied one after the other.
        const MAX_PASSES: usize = 10;

        let document = self.document(&params.path)?;

        let doc = &*document;
        let mut code = doc.get_document_content().to_string();

        let settings = self.workspaces();
        let Some(settings) = settings.settings_for(params.project_key, params.path.as_path())
        else {
            return Ok(FixFileResult {
                code,
                fixes_applied: 0,
                skipped_suggested_fixes: 0,
            });
        };

        let mut fixes_applied = 0;
        let mut skipped_suggested_fixes = 0;
        let mut converged = false;
        for _ in 0..MAX_PASSES {
            let doc = Document::new(code.clone(), 0);
            let diagnostics = self.lint_diagnostics(
                params.project_key,
                settings,
                &params.path,
                &doc,
                &params.only,
                &params.skip,
            );

            skipped_suggested_fixes = 0;
            let mut fixes: Vec<&RuleFix> = Vec::new();
            for fix in diagnostics.iter().filter_map(LinterDiagnostic::get_fix) {
                if fix.applicability == Applicability::Always
                    || params.fix_file_mode == FixFileMode::SafeAndUnsafeFixes
                {
                    fixes.push(fix);
                } else {
                    skipped_suggested_fixes += 1;
                }
            }
            fixes.sort_by_key(|fix| fix.range().map(TextRange::start));

            // Applies the fixes that don't touch the text of an earlier fix in this pass.
            let mut combined = RuleFix::new("");
            let mut applied = 0;
            for fix in fixes {
                let Some(range) = fix.range() else {
                    continue;
                };
                if combined
                    .range()
                    .is_some_and(|previous| range.start() <= previous.end())
                {
                    continue;
                }
                combined.edits.extend(fix.edits.iter().cloned());
                applied += 1;
            }
            if applied == 0 {
                converged = true;
                break;
            }
            code = combined.apply(&code);
            fixes_applied += applied;
        }
        if !converged {
            warn!(
                path = ?params.path,
                "Fixes still changed the file after {MAX_PASSES} passes, some may be left"
            );
        }

        Ok(FixFileResult {
            code,
            fixes_applied,
            skipped_suggested_fixes,
        })
    }

    #[cfg(feature = "db")]
    #[ignored_path(project=params.project_key, path=&params.path)]
    fn execute_statement(
        &self,
        params: ExecuteStatementParams,
    ) -> Result<ExecuteStatementResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let Some((_id, range, content, ast)) =
            doc.find(params.statement_id, ExecuteStatementMapper)
        else {
            return Ok(ExecuteStatementResult {
                message: "Statement was not found in document.".into(),
                ..Default::default()
            });
        };

        if ast.is_none() {
            return Ok(ExecuteStatementResult {
                message: "Statement is invalid.".into(),
                ..Default::default()
            });
        };

        let settings = self.workspaces();
        let no_connection = ExecuteStatementResult {
            message: "No database connection available.".into(),
            ..Default::default()
        };
        let Some(settings) = settings.settings_for(params.project_key, params.path.as_path())
        else {
            return Ok(no_connection);
        };

        if let Some(reason) = settings.db.statement_execution_disabled_reason() {
            return Ok(ExecuteStatementResult {
                message: reason,
                ..Default::default()
            });
        }

        let Some(result) = self.connection.with_pools(&settings.db, |pools| {
            let pool = pools.statement_pool.clone();
            run_async(async move { statement_execution::execute_statement(&pool, &content).await })?
                .map_err(WorkspaceError::from)
        }) else {
            return Ok(no_connection);
        };

        let mut result = result?;
        if let Some(position) = result
            .error
            .as_mut()
            .and_then(|error| error.position.as_mut())
        {
            *position += range.start();
        }
        Ok(result)
    }

    #[cfg(not(feature = "db"))]
    #[ignored_path(project=params.project_key, path=&params.path)]
    #[allow(unused_variables)]
    fn execute_statement(
        &self,
        params: ExecuteStatementParams,
    ) -> Result<ExecuteStatementResult, WorkspaceError> {
        Ok(ExecuteStatementResult {
            message: "Statement execution requires database support.".into(),
            ..Default::default()
        })
    }

    #[cfg(feature = "db")]
    fn invalidate_schema_cache(&self, all: bool) -> Result<(), WorkspaceError> {
        if all {
            // Clear all schemas - both db-loaded and json-loaded
            // DB completions always take precedence when a connection is available,
            // so clearing the json schema keeps behavior consistent
            self.snapshot.clear_all();
        } else {
            // Every project can have its own database, so clear the schemas of all connections
            self.snapshot.clear_all_connections();
        }
        Ok(())
    }

    #[cfg(not(feature = "db"))]
    fn invalidate_schema_cache(&self, _all: bool) -> Result<(), WorkspaceError> {
        self.snapshot.clear_all();
        Ok(())
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn pull_file_diagnostics(
        &self,
        params: PullFileDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        let settings = self.workspaces();

        let settings = match settings.settings_for(params.project_key, params.path.as_path()) {
            Some(settings) => settings,
            None => {
                // return an empty result if no settings are available
                // we might want to return an error here in the future
                return Ok(PullDiagnosticsResult {
                    diagnostics: Vec::new(),
                    skipped_diagnostics: 0,
                });
            }
        };

        let document = self.document(&params.path)?;

        let doc = &*document;

        /*
         * The statements in the document might already have associated diagnostics,
         * e.g. if they contain syntax errors that surfaced while parsing/splitting the statements
         */
        let mut diagnostics: Vec<SDiagnostic> = doc.document_diagnostics().to_vec();
        let path = params.path.as_path().display().to_string();

        if let Some(migrations) = &settings.migrations {
            diagnostics.extend(
                migration::section_marker_diagnostics(
                    doc.get_document_content(),
                    migrations.section_markers,
                    &document_sections(settings, doc),
                    doc.statement_ranges(),
                )
                .into_iter()
                .map(|diagnostic| SDiagnostic::new(Error::from(diagnostic).with_file_path(&path))),
            );
        }

        let should_stop = self.should_stop(&params.path, doc);
        let Some(analysis) = self.analyse_document(
            params.project_key,
            settings,
            &params.path,
            doc,
            AnalysisRequest {
                categories: params.categories,
                only: &params.only,
                skip: &params.skip,
                should_stop: &should_stop,
            },
        ) else {
            // The document changed or the request was cancelled. The diagnostics would be
            // outdated anyway.
            return Ok(PullDiagnosticsResult::default());
        };

        /*
         * Checks against the database: statements that reference only objects that exist
         * unchanged in the database are typechecked with EXPLAIN, and PL/pgSQL functions with
         * plpgsql_check.
         */
        #[cfg(feature = "db")]
        let plpgsql_checked = {
            let (db_diagnostics, plpgsql_checked) =
                self.database_diagnostics(settings, &params.path, doc, &analysis)?;
            diagnostics.extend(db_diagnostics);
            plpgsql_checked
        };
        #[cfg(not(feature = "db"))]
        let plpgsql_checked: Vec<TextRange> = Vec::new();

        for diagnostic in analysis.syntax_diagnostics() {
            // Ignore the syntax error if we already have more specialized diagnostics for the
            // same statement, e.g. from plpgsql_check for CREATE FUNCTION statements.
            let covered = diagnostics.iter().any(|d| {
                d.location().span.is_some_and(|span| {
                    diagnostic
                        .location()
                        .span
                        .is_some_and(|syntax_span| syntax_span.contains_range(span))
                })
            });
            // plpgsql_check compiles the function with the PL/pgSQL compiler and locates its
            // syntax errors, while the error of libpg_query spans the whole function body.
            let checked = diagnostic.location().span.is_some_and(|syntax_span| {
                plpgsql_checked
                    .iter()
                    .any(|range| range.contains_range(syntax_span))
            });
            if !covered && !checked {
                diagnostics.push(SDiagnostic::new(
                    diagnostic
                        .clone()
                        .with_file_path(path.clone())
                        .with_severity(Severity::Error),
                ));
            }
        }

        diagnostics.extend(
            analysis
                .lint_diagnostics()
                .iter()
                .cloned()
                .map(Error::from)
                .map(|d| {
                    let severity = d
                        .category()
                        .and_then(|category| settings.get_severity_from_rule_code(category))
                        .unwrap_or(Severity::Warning);

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
            suppressions.get_disabled_diagnostic_suppressions_as_errors(analysis.disabled_rules());

        let mut unused_suppression_errors =
            suppressions.get_unused_suppressions_as_errors(&diagnostics);
        unused_suppression_errors.retain(|error| !analysis.is_skipped(error.span.start()));

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

        self.charge_document_caches(&params.path, doc);
        info!("Pulled {:?} diagnostic(s)", diagnostics.len());
        Ok(PullDiagnosticsResult {
            diagnostics,
            skipped_diagnostics: 0,
        })
    }

    #[cfg(feature = "db")]
    fn pull_db_diagnostics(
        &self,
        params: crate::features::diagnostics::PullDatabaseDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        let settings = self.workspaces();
        let Some(settings) = settings.settings(params.project_key) else {
            debug!("No settings available. Returning empty diagnostics.");
            return Ok(PullDiagnosticsResult::default());
        };

        if !settings.splinter.enabled {
            debug!("Splinter is disabled. Skipping database linting.");
            return Ok(PullDiagnosticsResult::default());
        }

        let (enabled_rules, disabled_rules) = AnalyserVisitorBuilder::new(settings)
            .with_splinter_rules(&params.only, &params.skip)
            .finish();

        let categories = params.categories;
        let splinter_config = settings.splinter.to_configuration();
        let Some(splinter_diagnostics) = self.connection.with_pool(&settings.db, |pool| {
            let snapshot = match self.snapshot.load(pool) {
                Ok(snapshot) => Some(snapshot),
                Err(err @ WorkspaceError::DatabaseConnectionError(_)) => return Err(err),
                Err(err) => {
                    debug!("Unable to load the database snapshot for splinter: {err}");
                    None
                }
            };

            let pool = pool.clone();
            let splinter_result = run_async(async move {
                let filter = AnalysisFilter {
                    categories,
                    enabled_rules: Some(enabled_rules.as_slice()),
                    disabled_rules: &disabled_rules,
                };
                let splinter_params = pgls_splinter::SplinterParams {
                    conn: &pool,
                    snapshot: snapshot.as_deref(),
                    config: Some(&splinter_config),
                };
                pgls_splinter::run_splinter(splinter_params, &filter).await
            })?;

            splinter_result.map_err(WorkspaceError::from)
        }) else {
            debug!("No database connection available. Skipping splinter checks.");
            return Ok(PullDiagnosticsResult::default());
        };
        let splinter_diagnostics = splinter_diagnostics?;

        let total = splinter_diagnostics.len();
        let max = params.max_diagnostics as usize;
        let diagnostics: Vec<SDiagnostic> = splinter_diagnostics
            .into_iter()
            .take(max)
            .map(SDiagnostic::new)
            .collect();
        let skipped = total.saturating_sub(max) as u32;

        Ok(PullDiagnosticsResult {
            diagnostics,
            skipped_diagnostics: skipped,
        })
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn pull_file_formatting(
        &self,
        params: PullFileFormattingParams,
    ) -> Result<PullFormattingResult, WorkspaceError> {
        let settings = self.workspaces();

        let settings = match settings.settings_for(params.project_key, params.path.as_path()) {
            Some(settings) => settings,
            None => {
                return Ok(PullFormattingResult::default());
            }
        };

        if !settings.formatter.enabled {
            return Ok(PullFormattingResult::default());
        }

        let document = self.document(&params.path)?;

        let doc = &*document;

        let result = formatting::format_document(
            doc,
            &settings.formatter,
            params.range,
            &params.path.as_path().display().to_string(),
        );
        self.charge_document_caches(&params.path, doc);
        Ok(result)
    }

    #[cfg(feature = "db")]
    fn check_database_connection(
        &self,
        params: CheckDatabaseConnectionParams,
    ) -> Result<CheckDatabaseConnectionResult, WorkspaceError> {
        // Every configuration file of the project can have its own database, so check all of them.
        let settings = self.workspaces();
        for settings in settings.as_ref().get_all_settings(params.project_key) {
            if let Some(error) = &settings.db.connection_string_error {
                return Ok(CheckDatabaseConnectionResult {
                    error: Some(error.clone()),
                });
            }

            let result = self
                .connection
                .try_with_pool(&settings.db, |pool| self.snapshot.load(pool).map(|_| ()));
            match result {
                None | Some(Ok(())) => {}
                Some(Err(err @ WorkspaceError::DatabaseConnectionError(_))) => {
                    return Ok(CheckDatabaseConnectionResult {
                        error: Some(err.to_string()),
                    });
                }
                Some(Err(err)) => return Err(err),
            }
        }
        Ok(CheckDatabaseConnectionResult::default())
    }

    #[cfg(not(feature = "db"))]
    fn check_database_connection(
        &self,
        _params: CheckDatabaseConnectionParams,
    ) -> Result<CheckDatabaseConnectionResult, WorkspaceError> {
        Ok(CheckDatabaseConnectionResult::default())
    }

    #[cfg(not(feature = "db"))]
    fn pull_db_diagnostics(
        &self,
        _params: crate::features::diagnostics::PullDatabaseDiagnosticsParams,
    ) -> Result<PullDiagnosticsResult, WorkspaceError> {
        debug!("Database diagnostics require database support.");
        Ok(PullDiagnosticsResult::default())
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn get_completions(
        &self,
        params: GetCompletionsParams,
    ) -> Result<CompletionsResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());

        #[cfg(feature = "db")]
        let Some(snapshot) = settings
            .and_then(|settings| {
                self.connection
                    .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            })
            .transpose()?
        else {
            tracing::debug!("No database connection available. Skipping completions.");
            return Ok(CompletionsResult::default());
        };

        #[cfg(not(feature = "db"))]
        let Some(snapshot) = self.snapshot.get() else {
            tracing::debug!("No schema loaded. Skipping completions.");
            return Ok(CompletionsResult::default());
        };

        match get_statement_for_completions(doc, params.position) {
            None => {
                tracing::debug!("No statement found.");
                Ok(CompletionsResult::default())
            }
            Some((stmt, cst)) => {
                let position = stmt.statement_offset(params.position);
                let request =
                    self.editor_request(params.project_key, &params.path, settings, snapshot);
                let snapshot = self.snapshot_before(&request, doc, &stmt);

                let items = pgls_completions::complete(pgls_completions::CompletionParams {
                    position,
                    schema: snapshot.as_ref(),
                    tree: &cst,
                    text: stmt.id.content().to_string(),
                });

                self.charge_document_caches(&params.path, doc);
                Ok(CompletionsResult::in_document(items, &stmt))
            }
        }
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn on_hover(&self, params: OnHoverParams) -> Result<OnHoverResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());

        #[cfg(feature = "db")]
        let Some(snapshot) = settings
            .and_then(|settings| {
                self.connection
                    .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            })
            .transpose()?
        else {
            tracing::debug!("No database connection available. Skipping hover.");
            return Ok(OnHoverResult::default());
        };

        #[cfg(not(feature = "db"))]
        let Some(snapshot) = self.snapshot.get() else {
            tracing::debug!("No schema loaded. Skipping hover.");
            return Ok(OnHoverResult::default());
        };

        // The innermost statement: the body of a SQL function rather than its `CREATE FUNCTION`.
        match doc.iter_at(WithCSTMapper, params.position).last() {
            Some((stmt, ts_tree)) => {
                let position_in_stmt = stmt.statement_offset(params.position);
                let request =
                    self.editor_request(params.project_key, &params.path, settings, snapshot);
                let (references, snapshot) =
                    self.hover_references(&request, doc, &stmt, params.position);

                let markdown_blocks = pgls_hover::on_hover(pgls_hover::OnHoverParams {
                    ts_tree: &ts_tree,
                    snapshot: &snapshot,
                    position: position_in_stmt,
                    stmt_sql: stmt.id.content(),
                    references: &references,
                })
                .unwrap_or_default();

                self.charge_document_caches(&params.path, doc);
                Ok(OnHoverResult { markdown_blocks })
            }
            None => Ok(OnHoverResult::default()),
        }
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        range = ?params.range
    ), err)]
    fn pull_inlay_hints(
        &self,
        params: PullInlayHintsParams,
    ) -> Result<PullInlayHintsResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());
        if !settings.is_none_or(|settings| settings.inlay_hints.enabled) {
            return Ok(PullInlayHintsResult::default());
        }

        #[cfg(feature = "db")]
        let Some(snapshot) = settings
            .and_then(|settings| {
                self.connection
                    .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            })
            .transpose()?
        else {
            tracing::debug!("No database connection available. Skipping inlay hints.");
            return Ok(PullInlayHintsResult::default());
        };

        #[cfg(not(feature = "db"))]
        let Some(snapshot) = self.snapshot.get() else {
            tracing::debug!("No schema loaded. Skipping inlay hints.");
            return Ok(PullInlayHintsResult::default());
        };

        let request = self.editor_request(params.project_key, &params.path, settings, snapshot);
        let mut hints = Vec::new();
        for statements in self.resolved_statements(&request, doc, params.range) {
            for resolved in statements.iter() {
                let stmt = &resolved.stmt;
                let Some(sql) = stmt.sql.as_deref() else {
                    continue;
                };
                if stmt.range.intersect(params.range).is_none() {
                    continue;
                }
                hints.extend(
                    pgls_inlay_hints::inlay_hints_of_references(&resolved.references, sql)
                        .into_iter()
                        .map(|hint| InlayHint {
                            offset: stmt.document_offset(hint.offset),
                            label: hint.label,
                            kind: hint.kind.into(),
                        }),
                );
            }
        }

        hints.sort_by_key(|hint| hint.offset);
        self.charge_document_caches(&params.path, doc);
        Ok(PullInlayHintsResult { hints })
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        range = ?params.range
    ), err)]
    fn pull_semantic_tokens(
        &self,
        params: PullSemanticTokensParams,
    ) -> Result<PullSemanticTokensResult, WorkspaceError> {
        let document = self.document(&params.path)?;
        let doc = &*document;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());
        if !settings.is_none_or(|settings| settings.semantic_tokens.enabled) {
            return Ok(PullSemanticTokensResult::default());
        }

        #[cfg(feature = "db")]
        let Some(snapshot) = settings
            .and_then(|settings| {
                self.connection
                    .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            })
            .transpose()?
        else {
            tracing::debug!("No database connection available. Skipping semantic tokens.");
            return Ok(PullSemanticTokensResult::default());
        };

        #[cfg(not(feature = "db"))]
        let Some(snapshot) = self.snapshot.get() else {
            tracing::debug!("No schema loaded. Skipping semantic tokens.");
            return Ok(PullSemanticTokensResult::default());
        };

        let range = params
            .range
            .unwrap_or_else(|| TextRange::up_to(TextSize::of(doc.get_document_content())));
        let request = self.editor_request(params.project_key, &params.path, settings, snapshot);
        let mut tokens = Vec::new();
        for statements in self.resolved_statements(&request, doc, range) {
            for resolved in statements.iter() {
                let stmt = &resolved.stmt;
                let Some(sql) = stmt.sql.as_deref() else {
                    continue;
                };
                if stmt.range.intersect(range).is_none() {
                    continue;
                }
                tokens.extend(
                    pgls_semantic_tokens::semantic_tokens_of_references(&resolved.references, sql)
                        .into_iter()
                        .map(|token| SemanticToken {
                            range: stmt.document_range(token.range),
                            kind: token.kind.into(),
                            default_library: token.default_library,
                        }),
                );
            }
        }

        // The body of a SQL function is a statement of its own, inside its `CREATE FUNCTION`.
        tokens.sort_by_key(|token| (token.range.start(), token.range.end()));
        let mut end = TextSize::default();
        tokens.retain(|token| {
            let keep = token.range.start() >= end;
            if keep {
                end = token.range.end();
            }
            keep
        });
        self.charge_document_caches(&params.path, doc);
        Ok(PullSemanticTokensResult { tokens })
    }
}

/// The migration sections of `doc`, like the up and down migrations of dbmate and goose.
fn document_sections(settings: &Settings, doc: &Document) -> Vec<MigrationSection> {
    settings
        .migrations
        .as_ref()
        .map_or_else(Vec::new, |migrations| {
            migration::migration_sections(doc.get_document_content(), migrations.section_markers)
        })
}

/// The edit that adds `-- pgls-ignore <rule>` on its own line above the statement at `offset`,
/// with the statement's indentation and the file's line ending. Above the statement, the comment
/// suppresses the rule for all of the statement, including the bodies of its functions.
fn suppression_edit(doc: &Document, offset: TextSize, rule: &str) -> TextEdit {
    let content = doc.get_document_content();
    let start = doc
        .statement_ranges()
        .iter()
        .find(|statement| statement.contains_inclusive(offset))
        .map_or(offset, |statement| statement.start());
    let line_ending = match content.find('\n') {
        Some(newline) if content[..newline].ends_with('\r') => "\r\n",
        _ => "\n",
    };

    let before_start = &content[..usize::from(start)];
    let line_start = before_start.rfind('\n').map_or(0, |newline| newline + 1);
    let before = &before_start[line_start..];
    let indentation: String = before
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    if before.trim().is_empty() {
        TextEdit {
            range: TextRange::empty(start - TextSize::of(before)),
            new_text: format!("{indentation}-- pgls-ignore {rule}{line_ending}"),
        }
    } else {
        // The statement starts after another one on the same line, so it moves to a line of its
        // own.
        let previous_end = start - (TextSize::of(before) - TextSize::of(before.trim_end()));
        TextEdit {
            range: TextRange::new(previous_end, start),
            new_text: format!(
                "{line_ending}{indentation}-- pgls-ignore {rule}{line_ending}{indentation}"
            ),
        }
    }
}

#[cfg(all(test, feature = "db"))]
#[path = "server.tests.rs"]
mod tests;
