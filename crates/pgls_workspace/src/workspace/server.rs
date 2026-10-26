use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    panic::RefUnwindSafe,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use analyser::AnalyserVisitorBuilder;
#[cfg(feature = "db")]
use async_helper::run_async;
#[cfg(feature = "db")]
use connection_manager::ConnectionManager;
#[cfg(feature = "db")]
use document::TypecheckDiagnosticsMapper;
use document::{DefaultMapper, Document, ExecuteStatementMapper, FormatStatementMapper};
#[cfg(feature = "db")]
use futures::{StreamExt, TryStreamExt, stream};
#[cfg(feature = "db")]
use pg_query::convert_to_positional_params_with_metadata;
use pgls_analyse::{
    ActionCategory, AnalysisFilter, Applicability, RuleCategories, RuleCategoriesBuilder,
    RuleFilter, SUPPRESSION_ACTION_CATEGORY,
};
#[cfg(feature = "db")]
use pgls_analyser::StatementAnalysis;
use pgls_analyser::{
    Analyser, AnalyserConfig, AnalyserParams, AnalysisResult, Direction, FileKind,
    LinterDiagnostic, LinterOptions, MigrationSection, RuleFix,
};
use pgls_catalog::resolve::{ColumnSource, Reference, ResolveParams, resolve};
use pgls_catalog::{Catalog, CatalogBase, Origin, Session, TransactionMode};
use std::borrow::Cow;

use pgls_catalog::Snapshot;
use pgls_configuration::RuleSelector;
use pgls_diagnostics::{
    Diagnostic, DiagnosticExt, Error, Severity, serde::Diagnostic as SDiagnostic,
};
use pgls_fs::{ConfigName, PgLSPath};
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_text_size::{TextRange, TextSize};
#[cfg(feature = "db")]
use pgls_typecheck::TypecheckParams;
use pgls_workspace_macros::ignored_path;
use snapshot_manager::SnapshotManager;
use tracing::{debug, info, warn};

use crate::{
    WorkspaceError,
    configuration::to_analyser_rules,
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
        format::{PullFileFormattingParams, PullFormattingResult, StatementFormatResult},
        inlay_hints::{InlayHint, PullInlayHintsParams, PullInlayHintsResult},
        on_hover::{OnHoverParams, OnHoverResult},
        semantic_tokens::{PullSemanticTokensParams, PullSemanticTokensResult, SemanticToken},
    },
    settings::{Settings, WorkspaceSettings, WorkspaceSettingsHandle, WorkspaceSettingsHandleMut},
    workspace::{AnalyserDiagnosticsMapper, WithCSTandASTMapper},
};

use super::{
    FindProjectParams, GetFileContentParams, IsPathIgnoredParams, NestedSettingsError,
    OpenFileParams, ProjectKey, RegisterProjectFolderParams, ServerInfo,
    UnregisterProjectFolderParams, UpdateNestedSettingsParams, UpdateNestedSettingsResult,
    UpdateSettingsParams, Workspace,
};

pub use statement_identifier::StatementId;

mod analyser;
mod annotation;
#[cfg(feature = "db")]
mod async_helper;
#[cfg(feature = "db")]
mod connection_key;
#[cfg(feature = "db")]
use crate::settings::DatabaseSettings;
#[cfg(feature = "db")]
use connection_key::ConnectionKey;
#[cfg(feature = "db")]
mod connection_manager;
pub(crate) mod document;
mod expand_star;
mod migration;
mod pg_query;
#[cfg(feature = "db")]
mod pgpass;
mod snapshot_manager;
mod sql_function;
#[cfg(feature = "db")]
mod statement_execution;
mod statement_identifier;
mod tree_sitter;

pub struct WorkspaceServer {
    /// global settings object for this workspace
    settings: RwLock<WorkspaceSettings>,

    documents: RwLock<HashMap<PgLSPath, Document>>,

    /// The database snapshots, loaded from a database connection or from JSON
    snapshot: SnapshotManager,

    /// The indexed database snapshot the catalog starts from
    catalog_base: Mutex<Option<Arc<CatalogBase>>>,

    /// The last snapshot built for completions and hover
    snapshot_before: Mutex<Option<SnapshotBefore>>,

    #[cfg(feature = "db")]
    connection: ConnectionManager,
}

/// A database snapshot with the changes of the statements before a statement.
struct SnapshotBefore {
    /// The database snapshot it is built from.
    base: Arc<Snapshot>,
    /// The text that the statement sees.
    key: u64,
    snapshot: Arc<Snapshot>,
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
            snapshot_before: Mutex::default(),
            connection: ConnectionManager::new(),
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
            snapshot_before: Mutex::default(),
        }
    }

    /// Provides a reference to the current settings
    fn workspaces(&self) -> WorkspaceSettingsHandle<'_> {
        WorkspaceSettingsHandle::new(&self.settings)
    }

    fn workspaces_mut(&self) -> WorkspaceSettingsHandleMut<'_> {
        WorkspaceSettingsHandleMut::new(&self.settings)
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

    /// Whether the file is a migration. Migration-only rules don't run on other files.
    fn file_kind(&self, settings: &Settings, path: &Path) -> FileKind {
        match settings
            .migrations
            .as_ref()
            .and_then(|migrations| migrations.path.as_ref())
        {
            Some(migrations_dir) if migration::is_in_migrations_dir(path, migrations_dir) => {
                FileKind::Migration
            }
            Some(_) => FileKind::Other,
            None => FileKind::Unknown,
        }
    }

    /// Runs the lint and typecheck rules on a document. Returns the analysis, the syntax errors
    /// of the statements, the disabled rules, and the migration sections that are not linted.
    fn analyse_document(
        &self,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        categories: RuleCategories,
        only: &[RuleSelector],
        skip: &[RuleSelector],
    ) -> (
        AnalysisResult,
        Vec<SyntaxDiagnostic>,
        Vec<RuleFilter<'static>>,
        Vec<TextRange>,
    ) {
        #[cfg(feature = "db")]
        let snapshot = self
            .connection
            .with_pool(&settings.db, |pool| self.snapshot.load(pool))
            .and_then(Result::ok);

        #[cfg(not(feature = "db"))]
        let snapshot = self.snapshot.get();

        /*
         * Static analysis: the lint rules, and the typecheck rules when a database schema is
         * available. The catalog starts from the database schema and follows the DDL of the
         * file.
         */
        let typecheck = settings.typecheck.enabled && snapshot.is_some();
        let search_path = match snapshot.as_deref() {
            Some(snapshot) => snapshot.expand_search_path(&settings.typecheck.search_path),
            None => settings.typecheck.search_path.clone(),
        };
        let catalog_base = snapshot
            .as_ref()
            .map(|snapshot| self.catalog_base(snapshot));

        let (enabled_rules, disabled_rules) = AnalyserVisitorBuilder::new(settings)
            .with_linter_rules(only, skip)
            .finish();

        let options = LinterOptions {
            rules: to_analyser_rules(settings),
        };

        let filter = AnalysisFilter {
            categories,
            enabled_rules: Some(enabled_rules.as_slice()),
            disabled_rules: &disabled_rules,
        };

        let analyser = Analyser::new(AnalyserConfig {
            options: &options,
            filter,
        });

        /*
         * Migration tools like dbmate and goose put the up and the down migration in one file.
         * Each section is analysed as a migration of its own, and the down sections are only
         * checked with `lintDown`. Syntax errors are reported everywhere.
         */
        let sections = document_sections(settings, doc);
        let skipped_sections: Vec<TextRange> = sections
            .iter()
            .filter(|section| {
                section.direction == Direction::Down
                    && settings
                        .migrations
                        .as_ref()
                        .is_some_and(|migrations| !migrations.lint_down)
            })
            .map(|section| section.range)
            .collect();
        let is_skipped = |range: TextRange| {
            skipped_sections
                .iter()
                .any(|section| section.contains(range.start()))
        };

        let mut analysable_stmts = vec![];
        let mut syntax_diagnostics = vec![];
        let mut unparsable = vec![];
        for (stmt, diagnostic, unparsable_stmt) in doc.iter(AnalyserDiagnosticsMapper) {
            analysable_stmts.extend(stmt.filter(|stmt| !is_skipped(stmt.range)));
            syntax_diagnostics.extend(diagnostic);
            unparsable.extend(unparsable_stmt.filter(|stmt| !is_skipped(stmt.range)));
        }

        let analysis = analyser.analyse(AnalyserParams {
            stmts: analysable_stmts,
            unparsable,
            catalog_base,
            search_path,
            file_kind: self.file_kind(settings, path.as_path()),
            sections,
            typecheck,
        });

        (
            analysis,
            syntax_diagnostics,
            disabled_rules,
            skipped_sections,
        )
    }

    /// The fixes of the lint diagnostics at the cursor, and an action to suppress each of them.
    fn lint_code_actions(
        &self,
        settings: &Settings,
        params: &CodeActionsParams,
        doc: &Document,
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();
        for diagnostic in
            self.lint_diagnostics(settings, &params.path, doc, &params.only, &params.skip)
        {
            let Some(span) = diagnostic.location().span else {
                continue;
            };
            if !span.contains_inclusive(params.cursor_position) {
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
        let (stmt_start, references) = self.references_at(settings, doc, &snapshot, position);
        references
            .iter()
            .filter_map(|reference| expand_star::expand_star_action(reference, content, stmt_start))
            .collect()
    }

    /// The lint diagnostics of a document that are not suppressed.
    fn lint_diagnostics(
        &self,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        only: &[RuleSelector],
        skip: &[RuleSelector],
    ) -> Vec<LinterDiagnostic> {
        let (analysis, _, _, _) = self.analyse_document(
            settings,
            path,
            doc,
            RuleCategoriesBuilder::default().all().build(),
            only,
            skip,
        );
        let suppressions = doc.suppressions();
        analysis
            .diagnostics
            .into_iter()
            .filter(|diagnostic| !suppressions.is_suppressed(diagnostic))
            .collect()
    }

    /// Walks the statements of `doc` that start before `end` like the analyser does: the catalog
    /// and the session follow the statements, so that each statement sees the objects created
    /// earlier in the file. Each migration section gets a session of its own, and the changes of
    /// a down section don't carry into the next section. `visit` gets every parsed statement with
    /// the catalog and the session as they are before it. `settings` are the ones of the file.
    /// Returns the catalog after the walked statements.
    fn walk_statements(
        &self,
        settings: Option<&Settings>,
        doc: &Document,
        snapshot: &Arc<Snapshot>,
        end: TextSize,
        mut visit: impl FnMut(&pgls_analyser::AnalysableStatement, &Catalog, &Session),
    ) -> Catalog {
        let (search_path, sections) = settings
            .map(|settings| {
                (
                    settings.typecheck.search_path.clone(),
                    document_sections(settings, doc),
                )
            })
            .unwrap_or_default();
        let search_path = snapshot.expand_search_path(&search_path);
        let new_session = |section: Option<&MigrationSection>| {
            Session::new(search_path.clone()).with_transaction_mode(
                section.map_or(TransactionMode::Unknown, MigrationSection::transaction_mode),
            )
        };

        // Editor features show what names most likely refer to, so a `DO` block or a statement
        // that doesn't parse doesn't turn them off for the rest of the file.
        let mut catalog = Catalog::new(Some(self.catalog_base(snapshot))).best_effort();
        let mut session = new_session(None);
        let mut section: Option<&MigrationSection> = None;
        // The catalog after the up sections so far, while walking a down section.
        let mut up_catalog: Option<Catalog> = None;
        for (stmt, _, unparsable) in doc.iter(AnalyserDiagnosticsMapper) {
            let Some(start) = stmt.as_ref().map(|stmt| stmt.range.start()).or_else(|| {
                unparsable
                    .as_ref()
                    .map(|unparsable| unparsable.range.start())
            }) else {
                continue;
            };
            if start >= end {
                break;
            }
            let current = sections
                .iter()
                .find(|section| section.range.contains(start));
            if current != section {
                if let Some(up_catalog) = up_catalog.take() {
                    catalog = up_catalog;
                }
                if current.is_some_and(|section| section.direction == Direction::Down) {
                    up_catalog = Some(catalog.clone());
                }
                session = new_session(current);
                section = current;
            }

            if let Some(unparsable) = unparsable {
                if pgls_analyser::may_change_catalog(&unparsable.sql) {
                    catalog.taint();
                }
                continue;
            }
            let Some(stmt) = stmt else {
                continue;
            };
            // The names of statements with identifier parameters like `:name` are placeholders.
            if stmt.has_identifier_parameters {
                if !pgls_analyser::is_plain_query(&stmt.root) {
                    catalog.taint();
                }
                continue;
            }
            visit(&stmt, &catalog, &session);
            // Statements in the bodies of SQL functions are not executed.
            if stmt.function.is_none() {
                catalog.apply(&stmt.root, session.search_path());
                session.apply(&stmt.root);
            }
        }
        catalog
    }

    /// For hover on the statement `id` at `range`: the references whose span contains
    /// `position`, and the database snapshot as the statement sees it.
    fn hover_references(
        &self,
        settings: Option<&Settings>,
        doc: &Document,
        id: &StatementId,
        range: TextRange,
        position: TextSize,
        snapshot: Arc<Snapshot>,
    ) -> (Vec<Reference>, Arc<Snapshot>) {
        let search_path = settings
            .map(|settings| settings.typecheck.search_path.clone())
            .unwrap_or_default();
        let mut found = None;
        let catalog = self.walk_statements(
            settings,
            doc,
            &snapshot,
            position + TextSize::from(1),
            |stmt, catalog, session| {
                let Some(sql) = stmt.sql.as_deref() else {
                    return;
                };
                if stmt.range != range {
                    return;
                }
                let position = position - range.start();
                let (references, others): (Vec<_>, Vec<_>) = resolve(ResolveParams {
                    stmt: &stmt.root,
                    catalog,
                    search_path: session.search_path(),
                    function: stmt.function.as_ref(),
                    sql: Some(sql),
                    collect_references: true,
                })
                .references
                .into_iter()
                .partition(|reference| {
                    reference
                        .span()
                        .is_some_and(|span| span.contains_inclusive(position))
                });
                // Objects that the file didn't change are the same in the database snapshot.
                let snapshot = if in_database(&references, &others) {
                    Arc::clone(&snapshot)
                } else {
                    self.snapshot_before(
                        doc,
                        id,
                        range,
                        Arc::clone(&snapshot),
                        &search_path,
                        || catalog,
                    )
                };
                found = Some((references, snapshot));
            },
        );
        // A statement that doesn't parse is not visited. It changes nothing, so it sees the
        // catalog after the walk.
        found.unwrap_or_else(|| {
            (
                Vec::new(),
                self.snapshot_before(doc, id, range, snapshot, &search_path, || catalog),
            )
        })
    }

    /// The references whose span contains `position`, in the innermost statement at `position`
    /// (the body of a SQL function rather than its `CREATE FUNCTION`). Their spans are relative
    /// to the start of that statement, which is returned with them.
    fn references_at(
        &self,
        settings: Option<&Settings>,
        doc: &Document,
        snapshot: &Arc<Snapshot>,
        position: TextSize,
    ) -> (TextSize, Vec<Reference>) {
        let mut innermost: Option<(TextRange, Vec<Reference>)> = None;
        self.walk_statements(
            settings,
            doc,
            snapshot,
            position + TextSize::from(1),
            |stmt, catalog, session| {
                let Some(sql) = stmt.sql.as_deref() else {
                    return;
                };
                if !stmt.range.contains(position)
                    || innermost
                        .as_ref()
                        .is_some_and(|(range, _)| !range.contains_range(stmt.range))
                {
                    return;
                }
                let references = resolve(ResolveParams {
                    stmt: &stmt.root,
                    catalog,
                    search_path: session.search_path(),
                    function: stmt.function.as_ref(),
                    sql: Some(sql),
                    collect_references: true,
                })
                .references;
                innermost = Some((stmt.range, references));
            },
        );
        let Some((range, references)) = innermost else {
            return (TextSize::default(), Vec::new());
        };
        let position = position - range.start();
        let references = references
            .into_iter()
            .filter(|reference| {
                reference
                    .span()
                    .is_some_and(|span| span.contains_inclusive(position))
            })
            .collect();
        (range.start(), references)
    }

    /// The indexed database snapshot, built once per snapshot.
    fn catalog_base(&self, snapshot: &Arc<Snapshot>) -> Arc<CatalogBase> {
        let mut cached = self.catalog_base.lock().unwrap();
        if let Some(base) = cached.as_ref().filter(|base| base.is_built_from(snapshot)) {
            return Arc::clone(base);
        }
        let base = Arc::new(CatalogBase::new(Arc::clone(snapshot)));
        *cached = Some(Arc::clone(&base));
        base
    }

    /// The database snapshot as the statement `id` at `range` sees it: with the objects that the
    /// statements before it create, change, or drop. `catalog` returns the catalog that the
    /// statement sees, and is only called when the snapshot is not cached.
    fn snapshot_before<C: Borrow<Catalog>>(
        &self,
        doc: &Document,
        id: &StatementId,
        range: TextRange,
        snapshot: Arc<Snapshot>,
        search_path: &[String],
        catalog: impl FnOnce() -> C,
    ) -> Arc<Snapshot> {
        // The statement sees the text before it, and the body of a SQL function also sees its
        // `CREATE FUNCTION`. Files with other settings can see another search path.
        let mut hasher = DefaultHasher::new();
        doc.get_document_content()[..usize::from(range.start())].hash(&mut hasher);
        id.parent_content().hash(&mut hasher);
        search_path.hash(&mut hasher);
        let key = hasher.finish();

        let mut cached = self.snapshot_before.lock().unwrap();
        if let Some(cached) = cached.as_ref()
            && cached.key == key
            && Arc::ptr_eq(&cached.base, &snapshot)
        {
            return Arc::clone(&cached.snapshot);
        }

        let result = catalog()
            .borrow()
            .snapshot()
            .unwrap_or_else(|| Arc::clone(&snapshot));
        *cached = Some(SnapshotBefore {
            base: snapshot,
            key,
            snapshot: Arc::clone(&result),
        });
        result
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
        statements: &[StatementAnalysis],
        skipped_sections: &[TextRange],
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
                .filter(|(_, range, ..)| {
                    !skipped_sections
                        .iter()
                        .any(|section| section.contains(range.start()))
                })
                .map(|(id, range, ast, cst)| {
                    let statement = statements
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
        let mut workspace = self.workspaces_mut();
        let workspace = workspace.as_mut();
        let existing = workspace.get_project_key(&path);
        debug!("Register the project {:?}, known as {:?}", path, existing);

        let key = existing.unwrap_or_else(|| workspace.insert_project(path));
        workspace.set_real_path(key, params.real_path);
        Ok(key)
    }

    fn find_project(
        &self,
        params: FindProjectParams,
    ) -> Result<Option<ProjectKey>, WorkspaceError> {
        Ok(self
            .workspaces()
            .as_ref()
            .find_project(params.path.as_path()))
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
        let settings = workspace
            .as_mut()
            .get_settings_mut(params.project_key)
            .ok_or_else(WorkspaceError::no_project)?;

        if params.replace {
            // A configuration that can't be applied leaves the settings as they were.
            let mut replacement = Settings::default();
            replacement.merge_with_configuration(
                params.configuration,
                params.workspace_directory,
                params.vcs_base_path,
                params.gitignore_matches.as_slice(),
            )?;
            *settings = replacement;
        } else {
            settings.merge_with_configuration(
                params.configuration,
                params.workspace_directory,
                params.vcs_base_path,
                params.gitignore_matches.as_slice(),
            )?;
        }
        // The search path and the migration sections change what a statement sees.
        *self.snapshot_before.lock().unwrap() = None;

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
        // The search path and the migration sections change what a statement sees.
        *self.snapshot_before.lock().unwrap() = None;
        Ok(result)
    }

    /// Add a new file to the workspace
    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "info", skip_all, fields(path = params.path.as_path().as_os_str().to_str()), err)]
    fn open_file(&self, params: OpenFileParams) -> Result<(), WorkspaceError> {
        let mut documents = self.documents.write().unwrap();
        documents
            .entry(params.path.clone())
            .or_insert_with(|| Document::new(params.content, params.version));

        Ok(())
    }

    /// Remove a file from the workspace
    #[ignored_path(project=params.project_key, path=&params.path)]
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
    #[ignored_path(project=params.project_key, path=&params.path)]
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

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn get_file_content(&self, params: GetFileContentParams) -> Result<String, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let document = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;
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
        let documents = self.documents.read().unwrap();
        let parser = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let settings = self.workspaces();
        let settings = settings.settings_for(params.project_key, params.path.as_path());

        let disabled_reason = match settings {
            Some(settings) => settings.db.statement_execution_disabled_reason(),
            None => Some("Statement execution not allowed against database.".into()),
        };

        let mut actions: Vec<CodeAction> = parser
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
            self.refactor_code_actions(settings, parser, params.cursor_position),
        );

        if let Some(settings) = settings {
            let mut lint_actions = self.lint_code_actions(settings, &params, parser);
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

        Ok(CodeActionsResult { actions })
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    fn fix_file(&self, params: FixFileParams) -> Result<FixFileResult, WorkspaceError> {
        /// Fixes can enable other fixes, e.g. `CONCURRENTLY` enables `IF NOT EXISTS`, and fixes
        /// that overlap are applied one after the other.
        const MAX_PASSES: usize = 10;

        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;
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
            let diagnostics =
                self.lint_diagnostics(settings, &params.path, &doc, &params.only, &params.skip);

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
        let documents = self.documents.read().unwrap();
        let parser = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let stmt = parser.find(params.statement_id, ExecuteStatementMapper);

        if stmt.is_none() {
            return Ok(ExecuteStatementResult {
                message: "Statement was not found in document.".into(),
                ..Default::default()
            });
        };

        let (_id, range, content, ast) = stmt.unwrap();

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

        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

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

        let (analysis, syntax_diagnostics, disabled_rules, skipped_sections) = self
            .analyse_document(
                settings,
                &params.path,
                doc,
                params.categories,
                &params.only,
                &params.skip,
            );
        let is_skipped = |range: TextRange| {
            skipped_sections
                .iter()
                .any(|section| section.contains(range.start()))
        };

        /*
         * Checks against the database: statements that reference only objects that exist
         * unchanged in the database are typechecked with EXPLAIN, and PL/pgSQL functions with
         * plpgsql_check.
         */
        #[cfg(feature = "db")]
        let plpgsql_checked = {
            let (db_diagnostics, plpgsql_checked) = self.database_diagnostics(
                settings,
                &params.path,
                doc,
                &analysis.statements,
                &skipped_sections,
            )?;
            diagnostics.extend(db_diagnostics);
            plpgsql_checked
        };
        #[cfg(not(feature = "db"))]
        let plpgsql_checked: Vec<TextRange> = Vec::new();

        for diagnostic in syntax_diagnostics {
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
                        .with_file_path(path.clone())
                        .with_severity(Severity::Error),
                ));
            }
        }

        diagnostics.extend(analysis.diagnostics.into_iter().map(Error::from).map(|d| {
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
        }));

        let suppressions = doc.suppressions();

        let disabled_suppression_errors =
            suppressions.get_disabled_diagnostic_suppressions_as_errors(&disabled_rules);

        let mut unused_suppression_errors =
            suppressions.get_unused_suppressions_as_errors(&diagnostics);
        unused_suppression_errors.retain(|error| !is_skipped(error.span));

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

        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let config = pgls_pretty_print::FormatConfig {
            line_width: settings.formatter.line_width as usize,
            indent_size: settings.formatter.indent_size as usize,
            indent_style: settings.formatter.indent_style.into(),
            keyword_case: settings.formatter.keyword_case.into(),
            constant_case: settings.formatter.constant_case.into(),
            type_case: settings.formatter.type_case.into(),
            comma_style: settings.formatter.comma_style.into(),
            logical_operator_placement: settings.formatter.logical_operator_placement.into(),
            layout: settings.formatter.layout.into(),
            values_row_layout: settings.formatter.values_row_layout.into(),
            cast_style: settings.formatter.cast_style.into(),
            clause_body_style: settings.formatter.clause_body_style.into(),
            isolate_semicolon: settings.formatter.isolate_semicolon,
            function_argument_groups: settings
                .formatter
                .function_argument_groups
                .iter()
                .map(|(name, size)| (name.to_lowercase(), usize::from(size.get())))
                .collect(),
        };

        let mut diagnostics = Vec::new();
        let mut statements = Vec::new();
        let mut formatted_output = String::new();
        let path_str = params.path.as_path().display().to_string();

        let format_candidates: Vec<_> = doc.iter(FormatStatementMapper).collect();
        let mut formatted_sql_fn_bodies = HashMap::new();

        // Only format SQL function bodies if skip_fn_bodies is false. A body with several
        // statements is left as written.
        if !settings.formatter.skip_fn_bodies {
            let mut statements_per_body: HashMap<StatementId, usize> = HashMap::new();
            for (id, ..) in &format_candidates {
                if let Some(parent_id) = id.parent() {
                    *statements_per_body.entry(parent_id).or_default() += 1;
                }
            }
            for (id, _stmt_range, text, ast_result) in &format_candidates {
                if !id.is_child() {
                    continue;
                }

                let Some(parent_id) = id.parent() else {
                    continue;
                };

                if statements_per_body.get(&parent_id) != Some(&1) {
                    continue;
                }

                let Ok(ast) = ast_result else {
                    continue;
                };

                let Ok(result) = pgls_pretty_print::format_statement(ast, text, &config) else {
                    continue;
                };

                let Some(formatted) = restore_named_parameters(text, &result.formatted, ast) else {
                    continue;
                };

                formatted_sql_fn_bodies.insert(parent_id, formatted);
            }
        }

        // Reconstruct the file by replacing each statement's source span with its
        // formatted output while copying the gaps between statements (whitespace and
        // standalone comments) verbatim. libpg_query drops comments from the AST, so
        // anything that lives outside a statement's range would otherwise be lost.
        let source = doc.get_document_content();
        let mut last_end = 0usize;

        // Formatting a range leaves everything outside its statements as written.
        let collapses_blank_lines = params.range.is_none();

        for (id, stmt_range, text, ast_result) in format_candidates {
            if id.is_child() {
                continue;
            }

            let stmt_start = usize::from(stmt_range.start());
            let stmt_end = usize::from(stmt_range.end());

            // Preserve the standalone comments that precede this statement, with at most one
            // blank line between statements and none before the first.
            let gap = &source[last_end..stmt_start];
            if collapses_blank_lines {
                let gap = collapse_blank_lines(gap);
                formatted_output.push_str(if last_end == 0 {
                    gap.trim_start()
                } else {
                    &gap
                });
            } else {
                formatted_output.push_str(gap);
            }
            last_end = stmt_end;

            // Leave statements outside the requested range untouched.
            if let Some(filter_range) = params.range
                && stmt_range.intersect(filter_range).is_none()
            {
                formatted_output.push_str(&text);
                continue;
            }

            match ast_result {
                Ok(ast) => {
                    let mut ast = ast;
                    if let Some(formatted_sql_fn_body) = formatted_sql_fn_bodies.get(&id) {
                        sql_function::set_formatted_sql_fn_body(&mut ast, formatted_sql_fn_body);
                    }

                    match pgls_pretty_print::format_statement(&ast, &text, &config) {
                        Ok(result) => {
                            match restore_named_parameters(&text, &result.formatted, &ast) {
                                Some(formatted) => {
                                    if text != formatted {
                                        statements.push(StatementFormatResult {
                                            original: text.clone(),
                                            formatted: formatted.clone(),
                                            range: stmt_range,
                                        });
                                    }
                                    formatted_output.push_str(&formatted);
                                }
                                None => {
                                    diagnostics.push(SDiagnostic::new(
                                        pgls_diagnostics::Error::from(
                                            WorkspaceError::format_error(
                                                "Named parameters could not be restored after \
                                                 formatting, the statement was left untouched"
                                                    .to_string(),
                                            ),
                                        )
                                        .with_file_path(&path_str)
                                        .with_file_span(stmt_range),
                                    ));

                                    formatted_output.push_str(&text);
                                }
                            }
                        }
                        Err(err) => {
                            diagnostics.push(SDiagnostic::new(
                                pgls_diagnostics::Error::from(WorkspaceError::format_error(
                                    err.to_string(),
                                ))
                                .with_file_path(&path_str)
                                .with_file_span(stmt_range),
                            ));

                            formatted_output.push_str(&text);
                        }
                    }
                }
                Err(syntax_err) => {
                    diagnostics.push(SDiagnostic::new(
                        pgls_diagnostics::Error::from(syntax_err).with_file_path(&path_str),
                    ));

                    formatted_output.push_str(&text);
                }
            }
        }

        // Preserve any trailing text or comments after the last statement, without blank lines
        // at the end of the file but with its final line break.
        let rest = &source[last_end..];
        if collapses_blank_lines {
            let rest = collapse_blank_lines(rest);
            let rest = if last_end == 0 {
                rest.trim()
            } else {
                rest.trim_end()
            };
            formatted_output.push_str(rest);
            if source.ends_with('\n') && !formatted_output.is_empty() {
                formatted_output.push('\n');
            }
        } else {
            formatted_output.push_str(rest);
        }

        Ok(PullFormattingResult {
            original: doc.get_document_content().to_string(),
            formatted: formatted_output,
            statements,
            diagnostics,
        })
    }

    #[cfg(feature = "db")]
    fn check_database_connection(
        &self,
        params: CheckDatabaseConnectionParams,
    ) -> Result<CheckDatabaseConnectionResult, WorkspaceError> {
        // Every configuration file of the project can have its own database, so check each of
        // them once, with the folders that use it.
        let mut connections: Vec<(DatabaseSettings, Vec<PathBuf>)> = Vec::new();
        let mut errors = Vec::new();
        for (folder, settings) in self
            .workspaces()
            .as_ref()
            .get_all_settings(params.project_key)
        {
            // An invalid connection string disables the connection, so say why.
            if let Some(error) = &settings.db.connection_string_error {
                if !errors.contains(error) {
                    errors.push(error.clone());
                }
                continue;
            }
            if !settings.db.enable_connection {
                continue;
            }
            let existing = connections.iter_mut().find(|(db, _)| {
                ConnectionKey::from(db) == ConnectionKey::from(&settings.db)
                    && db.connection_string == settings.db.connection_string
                    && db.password == settings.db.password
                    && db.conn_timeout_secs == settings.db.conn_timeout_secs
            });
            match existing {
                Some((_, folders)) => folders.push(folder.to_path_buf()),
                None => connections.push((settings.db.clone(), vec![folder.to_path_buf()])),
            }
        }

        // Concurrently, so that unreachable databases time out together rather than one after
        // the other.
        let results: Vec<_> = std::thread::scope(|scope| {
            let checks: Vec<_> = connections
                .iter()
                .map(|(db, _)| {
                    scope.spawn(|| {
                        self.connection
                            .try_with_pool(db, |pool| self.snapshot.load(pool).map(|_| ()))
                    })
                })
                .collect();
            checks
                .into_iter()
                .map(|check| check.join().expect("a connection check panicked"))
                .collect()
        });

        for ((db, folders), result) in connections.iter().zip(results) {
            match result {
                None | Some(Ok(())) => {}
                Some(Err(err @ WorkspaceError::DatabaseConnectionError(_))) => {
                    let folders: Vec<_> = folders
                        .iter()
                        .filter(|folder| !folder.as_os_str().is_empty())
                        .map(|folder| folder.display().to_string())
                        .collect();
                    let used_by = if folders.is_empty() {
                        String::new()
                    } else {
                        format!(", the database of {}", folders.join(", "))
                    };
                    errors.push(format!(
                        "Could not connect to {}@{}:{}/{}{used_by}: {err}",
                        db.username, db.host, db.port, db.database
                    ));
                }
                Some(Err(err)) => return Err(err),
            }
        }
        Ok(CheckDatabaseConnectionResult { errors })
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
        let documents = self.documents.read().unwrap();
        let parsed_doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

        let settings = self.workspaces();
        let search_path = settings
            .settings_for(params.project_key, params.path.as_path())
            .map(|settings| settings.typecheck.search_path.clone())
            .unwrap_or_default();

        #[cfg(feature = "db")]
        let Some(snapshot) = settings
            .settings_for(params.project_key, params.path.as_path())
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

        match get_statement_for_completions(parsed_doc, params.position) {
            None => {
                tracing::debug!("No statement found.");
                Ok(CompletionsResult::default())
            }
            Some((id, range, cst)) => {
                let position = params.position - range.start();
                let snapshot = self.snapshot_before(
                    parsed_doc,
                    &id,
                    range,
                    Arc::clone(&snapshot),
                    &search_path,
                    || {
                        self.walk_statements(
                            settings.settings_for(params.project_key, params.path.as_path()),
                            parsed_doc,
                            &snapshot,
                            range.start(),
                            |_, _, _| {},
                        )
                    },
                );

                let items = pgls_completions::complete(pgls_completions::CompletionParams {
                    position,
                    schema: snapshot.as_ref(),
                    tree: &cst,
                    text: id.content().to_string(),
                });

                Ok(CompletionsResult::with_offset(items, range.start()))
            }
        }
    }

    #[ignored_path(project=params.project_key, path=&params.path)]
    #[tracing::instrument(level = "debug", skip_all, fields(
        path = params.path.as_os_str().to_str(),
        position = params.position.to_string()
    ), err)]
    fn on_hover(&self, params: OnHoverParams) -> Result<OnHoverResult, WorkspaceError> {
        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

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
        match doc.iter_at(WithCSTandASTMapper, params.position).last() {
            Some((stmt_id, range, ts_tree, maybe_ast)) => {
                let position_in_stmt = params.position - range.start();
                let (references, snapshot) = self.hover_references(
                    settings,
                    doc,
                    &stmt_id,
                    range,
                    params.position,
                    snapshot,
                );

                let markdown_blocks = pgls_hover::on_hover(pgls_hover::OnHoverParams {
                    ts_tree: &ts_tree,
                    snapshot: &snapshot,
                    ast: maybe_ast.as_ref(),
                    position: position_in_stmt,
                    stmt_sql: stmt_id.content(),
                    references: &references,
                })
                .unwrap_or_default();

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
        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

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

        let mut hints = Vec::new();
        self.walk_statements(
            settings,
            doc,
            &snapshot,
            params.range.end(),
            |stmt, catalog, session| {
                let Some(sql) = stmt.sql.as_deref() else {
                    return;
                };
                if stmt.range.intersect(params.range).is_none() {
                    return;
                }
                hints.extend(
                    pgls_inlay_hints::inlay_hints(pgls_inlay_hints::InlayHintsParams {
                        stmt: &stmt.root,
                        sql,
                        catalog,
                        search_path: session.search_path(),
                        function: stmt.function.as_ref(),
                    })
                    .into_iter()
                    .map(|hint| InlayHint {
                        offset: hint.offset + stmt.range.start(),
                        label: hint.label,
                        kind: hint.kind.into(),
                    }),
                );
            },
        );

        hints.sort_by_key(|hint| hint.offset);
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
        let documents = self.documents.read().unwrap();
        let doc = documents
            .get(&params.path)
            .ok_or(WorkspaceError::not_found())?;

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
        let mut tokens = Vec::new();
        self.walk_statements(
            settings,
            doc,
            &snapshot,
            range.end(),
            |stmt, catalog, session| {
                let Some(sql) = stmt.sql.as_deref() else {
                    return;
                };
                if stmt.range.intersect(range).is_none() {
                    return;
                }
                tokens.extend(
                    pgls_semantic_tokens::semantic_tokens(
                        pgls_semantic_tokens::SemanticTokensParams {
                            stmt: &stmt.root,
                            sql,
                            catalog,
                            search_path: session.search_path(),
                            function: stmt.function.as_ref(),
                        },
                    )
                    .into_iter()
                    .map(|token| SemanticToken {
                        range: token.range + stmt.range.start(),
                        kind: token.kind.into(),
                        default_library: token.default_library,
                    }),
                );
            },
        );

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
        Ok(PullSemanticTokensResult { tokens })
    }
}

/// Whether hover finds what `references` resolve to in the database snapshot: objects that the
/// file didn't change, and columns of CTEs and subqueries. `others` are the other references of
/// the statement.
fn in_database(references: &[Reference], others: &[Reference]) -> bool {
    // The statement references each relation that it takes columns from.
    let unchanged_relation = |schema: &str, name: &str| {
        references.iter().chain(others).any(|reference| {
            matches!(
                reference,
                Reference::Relation { relation, .. }
                    if relation.schema == schema
                        && relation.name == name
                        && relation.origin == Origin::Database
            )
        })
    };
    !references.is_empty()
        && references.iter().all(|reference| match reference {
            Reference::Relation { relation, .. } => relation.origin == Origin::Database,
            Reference::Function { function, .. } => function.origin == Origin::Database,
            Reference::Column {
                source: ColumnSource::Relation { schema, name },
                ..
            } => unchanged_relation(schema, name),
            Reference::Column { .. } => true,
            Reference::Type { type_info, .. } | Reference::EnumLiteral { type_info, .. } => {
                type_info.origin == Origin::Database
            }
            // Parameters belong to a function of the file.
            Reference::Parameter { .. } => false,
            // The columns of a `*` can come from objects that the file changed.
            Reference::Star { .. } => false,
        })
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

    let start = usize::from(start);
    let line_start = content[..start]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let before = &content[line_start..start];
    let indentation: String = before
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let offset = |offset: usize| TextSize::try_from(offset).unwrap();
    if before.trim().is_empty() {
        TextEdit {
            range: TextRange::empty(offset(line_start)),
            new_text: format!("{indentation}-- pgls-ignore {rule}{line_ending}"),
        }
    } else {
        // The statement starts after another one on the same line, so it moves to a line of its
        // own.
        let previous_end = line_start + before.trim_end().len();
        TextEdit {
            range: TextRange::new(offset(previous_end), offset(start)),
            new_text: format!(
                "{line_ending}{indentation}-- pgls-ignore {rule}{line_ending}{indentation}"
            ),
        }
    }
}

/// Returns `true` if `path` is a directory or
/// if it is a symlink that resolves to a directory.
/// Keeps at most one blank line in `text`, which only holds whitespace and comments.
fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    // Line breaks since the last text that is not whitespace. Two make a blank line.
    let mut line_breaks = 0;
    for line in text.split_inclusive('\n') {
        let is_blank = line.trim().is_empty();
        let ends_line = line.ends_with('\n');
        if is_blank && ends_line {
            line_breaks += 1;
            if line_breaks <= 2 {
                result.push('\n');
            }
            continue;
        }

        result.push_str(line);
        if !is_blank {
            line_breaks = usize::from(ends_line);
        }
    }
    result
}

/// Put psql named parameters back where the formatter printed their placeholders.
///
/// The parser only ever sees placeholders, so the printed statement carries `$1` or a bare
/// identifier instead of `:'agen_code'` or `:raw_data`. Substitution happens on the token stream
/// so that a `$1` inside a string literal is left alone, and the result is proven by converting it
/// back and comparing the normalized ASTs: a statement that does not survive that round trip is
/// reported as unrestorable and kept as it was written.
fn restore_named_parameters(
    original: &str,
    formatted: &str,
    original_ast: &pgls_query::NodeEnum,
) -> Option<String> {
    use pgls_lexer::{SyntaxKind, lex};

    let conversion = pgls_lexer::convert_to_positional_params_with_metadata(original);

    if conversion.identifier_replacements.is_empty() && conversion.value_replacements.is_empty() {
        return Some(formatted.to_string());
    }

    if !conversion.restorable {
        return None;
    }

    let lexed = lex(formatted);
    let mut restored = String::with_capacity(formatted.len());

    for (idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::EOF {
            break;
        }

        let text = lexed.text(idx);

        let replacement = match kind {
            SyntaxKind::IDENT => conversion.identifier_replacements.get(text),
            SyntaxKind::POSITIONAL_PARAM => text
                .trim_start_matches('$')
                .parse::<usize>()
                .ok()
                .and_then(|index| conversion.value_replacements.get(&index)),
            _ => None,
        };

        match replacement {
            Some(source_text) => restored.push_str(source_text),
            None => restored.push_str(text),
        }
    }

    let reparsed = pgls_query::parse(&pgls_lexer::convert_to_positional_params(&restored)).ok()?;
    let mut restored_ast = reparsed.into_root()?;
    let mut expected_ast = original_ast.clone();
    pgls_pretty_print::normalize_ast(&mut restored_ast);
    pgls_pretty_print::normalize_ast(&mut expected_ast);

    if restored_ast == expected_ast {
        Some(restored)
    } else {
        None
    }
}

#[cfg(all(test, feature = "db"))]
#[path = "server.tests.rs"]
mod tests;
