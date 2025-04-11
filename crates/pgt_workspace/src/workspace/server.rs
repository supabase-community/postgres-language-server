use std::{fs, panic::RefUnwindSafe, path::Path, sync::RwLock};

use analyser::AnalyserVisitorBuilder;
use async_helper::run_async;
use dashmap::DashMap;
use db_connection::DbConnection;
use document::Document;
use futures::{StreamExt, stream};
use parsed_document::{
    AsyncDiagnosticsMapper, CursorPositionFilter, DefaultMapper, ExecuteStatementMapper,
    GetCompletionsMapper, ParsedDocument, SyncDiagnosticsMapper,
};
use pgt_analyse::{AnalyserOptions, AnalysisFilter};
use pgt_analyser::{Analyser, AnalyserConfig, AnalyserContext};
use pgt_diagnostics::{
    Diagnostic, DiagnosticExt, Error, Severity, serde::Diagnostic as SDiagnostic,
};
use pgt_fs::{ConfigName, PgTPath};
use pgt_typecheck::TypecheckParams;
use schema_cache_manager::SchemaCacheManager;
use sqlx::Executor;
use tracing::info;

use crate::{
    WorkspaceError,
    configuration::to_analyser_rules,
    features::{
        code_actions::{
            self, CodeAction, CodeActionKind, CodeActionsResult, CommandAction,
            CommandActionCategory, ExecuteStatementParams, ExecuteStatementResult,
        },
        completions::{CompletionsResult, GetCompletionsParams},
        diagnostics::{PullDiagnosticsParams, PullDiagnosticsResult},
    },
    settings::{Settings, SettingsHandle, SettingsHandleMut},
};

use super::{
    GetFileContentParams, IsPathIgnoredParams, OpenFileParams, ServerInfo, UpdateSettingsParams,
    Workspace,
};

pub use statement_identifier::StatementId;

mod analyser;
mod async_helper;
mod change;
mod db_connection;
mod document;
mod migration;
mod parsed_document;
mod pg_query;
mod schema_cache_manager;
mod sql_function;
mod statement_identifier;
mod tree_sitter;

pub(super) struct WorkspaceServer {
    /// global settings object for this workspace
    settings: RwLock<Settings>,

    /// Stores the schema cache for this workspace
    schema_cache: SchemaCacheManager,

    parsed_documents: DashMap<PgTPath, ParsedDocument>,

    connection: RwLock<DbConnection>,
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
            schema_cache: SchemaCacheManager::default(),
            connection: RwLock::default(),
        }
    }

    /// Provides a reference to the current settings
    fn settings(&self) -> SettingsHandle {
        SettingsHandle::new(&self.settings)
    }

    fn settings_mut(&self) -> SettingsHandleMut {
        SettingsHandleMut::new(&self.settings)
    }

    fn is_ignored_by_migration_config(&self, path: &Path) -> bool {
        let set = self.settings();
        set.as_ref()
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
        let set = self.settings();
        let settings = set.as_ref();
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
    /// Update the global settings for this workspace
    ///
    /// ## Panics
    /// This function may panic if the internal settings mutex has been poisoned
    /// by another thread having previously panicked while holding the lock
    #[tracing::instrument(level = "trace", skip(self), err)]
    fn update_settings(&self, params: UpdateSettingsParams) -> Result<(), WorkspaceError> {
        tracing::info!("Updating settings in workspace");

        self.settings_mut().as_mut().merge_with_configuration(
            params.configuration,
            params.workspace_directory,
            params.vcs_base_path,
            params.gitignore_matches.as_slice(),
        )?;

        tracing::info!("Updated settings in workspace");

        if !params.skip_db {
            self.connection
                .write()
                .unwrap()
                .set_conn_settings(&self.settings().as_ref().db);
        }

        tracing::info!("Updated Db connection settings");

        Ok(())
    }

    /// Add a new file to the workspace
    #[tracing::instrument(level = "info", skip_all, fields(path = params.path.as_path().as_os_str().to_str()), err)]
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError> {
        self.parsed_stmts_by_path
            .entry(params.path.clone())
            .or_insert_with(|| {
                ParsedDocument::new(params.path.clone(), params.content, params.version)
            });

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
        let mut parser = self
            .parsed_stmts_by_path
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

        let settings = self
            .settings
            .read()
            .expect("Unable to read settings for Code Actions");

        let disabled_reason: Option<String> = if settings.db.allow_statement_executions {
            None
        } else {
            Some("Statement execution not allowed against database.".into())
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

        let conn = self.connection.read().unwrap();
        let pool = match conn.get_pool() {
            Some(p) => p,
            None => {
                return Ok(ExecuteStatementResult {
                    message: "Not connected to database.".into(),
                });
            }
        };

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
        let settings = self.settings();

        // create analyser for this run
        // first, collect enabled and disabled rules from the workspace settings
        let (enabled_rules, disabled_rules) = AnalyserVisitorBuilder::new(settings.as_ref())
            .with_linter_rules(&params.only, &params.skip)
            .finish();
        // then, build a map that contains all options
        let options = AnalyserOptions {
            rules: to_analyser_rules(settings.as_ref()),
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

        // TODO: run this in parallel with rayon based on rayon.count()

        if let Some(pool) = self
            .connection
            .read()
            .expect("DbConnection RwLock panicked")
            .get_pool()
        {
            let path_clone = params.path.clone();
            let input = parser.iter(AsyncDiagnosticsMapper).collect::<Vec<_>>();
            let async_results = run_async(async move {
                stream::iter(input)
                    .map(|(_id, range, content, ast, cst)| {
                        let pool = pool.clone();
                        let path = path_clone.clone();
                        async move {
                            if let Some(ast) = ast {
                                pgt_typecheck::check_sql(TypecheckParams {
                                    conn: &pool,
                                    sql: &content,
                                    ast: &ast,
                                    tree: &cst,
                                })
                                .await
                                .map(|d| {
                                    let r = d.location().span.map(|span| span + range.start());

                                    d.with_file_path(path.as_path().display().to_string())
                                        .with_file_span(r.unwrap_or(range))
                                })
                            } else {
                                None
                            }
                        }
                    })
                    .buffer_unordered(10)
                    .collect::<Vec<_>>()
                    .await
            })?;

            for result in async_results.into_iter().flatten() {
                diagnostics.push(SDiagnostic::new(result));
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
                                        .as_ref()
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
        let parser = self
            .parsed_stmts_by_path
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let pool = match self.connection.read().unwrap().get_pool() {
            Some(pool) => pool,
            None => return Ok(CompletionsResult::default()),
        };

        let schema_cache = self.schema_cache.load(pool)?;

        let items = parser
            .iter_with_filter(
                GetCompletionsMapper,
                CursorPositionFilter::new(params.position),
            )
            .flat_map(|(_id, range, content, cst)| {
                // `offset` is the position in the document,
                // but we need the position within the *statement*.
                let position = params.position - range.start();
                pgt_completions::complete(pgt_completions::CompletionParams {
                    position,
                    schema: schema_cache.as_ref(),
                    tree: &cst,
                    text: content,
                })
            })
            .collect();

        Ok(CompletionsResult { items })
    }
}

/// Returns `true` if `path` is a directory or
/// if it is a symlink that resolves to a directory.
fn is_dir(path: &Path) -> bool {
    path.is_dir() || (path.is_symlink() && fs::read_link(path).is_ok_and(|path| path.is_dir()))
}
