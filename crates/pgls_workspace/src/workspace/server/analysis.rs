//! The lint and typecheck analysis of a document.

use std::{path::Path, sync::Arc};

use pgls_analyse::{AnalysisFilter, RuleCategories, RuleCategoriesBuilder, RuleFilter};
use pgls_analyser::{
    Analyser, AnalyserConfig, AnalyserParams, AnalysisResult, Direction, FileKind,
    LinterDiagnostic, LinterOptions, STOP_CHECK_INTERVAL,
};
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_text_size::{TextRange, TextSize};

use super::{
    WorkspaceServer, analyser::AnalyserVisitorBuilder, document::AnalyserDiagnosticsMapper,
    document::Document, document_sections, migration,
};
use crate::{configuration::to_analyser_rules, settings::Settings, workspace::ProjectKey};

/// What the analyser found in a document, with what the diagnostics need to be reported.
pub(crate) struct DocumentAnalysis {
    result: AnalysisResult,
    /// The syntax errors of the statements, including the bodies of SQL functions.
    syntax_diagnostics: Vec<SyntaxDiagnostic>,
    /// The rules that the settings or the request turned off.
    disabled_rules: Vec<RuleFilter<'static>>,
    /// The migration sections that are not linted: the down sections without `lintDown`.
    skipped_sections: Vec<TextRange>,
}

impl DocumentAnalysis {
    /// The diagnostics of the lint and typecheck rules, suppressed ones included.
    pub(crate) fn lint_diagnostics(&self) -> &[LinterDiagnostic] {
        &self.result.diagnostics
    }

    /// What the analyser learned about each statement.
    #[cfg(feature = "db")]
    pub(crate) fn statements(&self) -> &[pgls_analyser::StatementAnalysis] {
        &self.result.statements
    }

    pub(crate) fn syntax_diagnostics(&self) -> &[SyntaxDiagnostic] {
        &self.syntax_diagnostics
    }

    pub(crate) fn disabled_rules(&self) -> &[RuleFilter<'static>] {
        &self.disabled_rules
    }

    /// Whether `offset` is in a migration section that isn't linted.
    pub(crate) fn is_skipped(&self, offset: TextSize) -> bool {
        self.skipped_sections
            .iter()
            .any(|section| section.contains(offset))
    }
}

/// Which rules a request runs, and when it stops.
pub(super) struct AnalysisRequest<'a> {
    pub(super) categories: RuleCategories,
    pub(super) only: &'a [RuleSelector],
    pub(super) skip: &'a [RuleSelector],
    /// Checked while analysing, see [`Analyser::analyse_until`].
    pub(super) should_stop: &'a dyn Fn() -> bool,
}

impl<'a> AnalysisRequest<'a> {
    /// Every rule category, with the rules that `only` and `skip` select.
    pub(super) fn all(
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
        should_stop: &'a dyn Fn() -> bool,
    ) -> Self {
        Self {
            categories: RuleCategoriesBuilder::default().all().build(),
            only,
            skip,
            should_stop,
        }
    }
}

impl WorkspaceServer {
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

    /// Runs the lint and typecheck rules that `request` asks for on a document. `None` if the
    /// request stopped it.
    ///
    /// The analysis that the editor asks for after every edit and on every code action request,
    /// with every category and no rule selection, is kept in the document's cache until its
    /// content, the settings, or the database snapshot change.
    pub(super) fn analyse_document(
        &self,
        project_key: ProjectKey,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        request: AnalysisRequest,
    ) -> Option<Arc<DocumentAnalysis>> {
        let snapshot = self.database_snapshot(settings);
        let is_default = request.categories == RuleCategories::all()
            && request.only.is_empty()
            && request.skip.is_empty();
        if !is_default {
            return self
                .analyse(settings, path, doc, snapshot, &request)
                .map(Arc::new);
        }
        let key = self.cache_key(project_key, snapshot.as_ref());
        doc.cache().analysis(&key, doc.generation(), || {
            self.analyse(settings, path, doc, snapshot, &request)
        })
    }

    fn analyse(
        &self,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        snapshot: Option<Arc<pgls_catalog::Snapshot>>,
        request: &AnalysisRequest,
    ) -> Option<DocumentAnalysis> {
        let AnalysisRequest {
            categories,
            only,
            skip,
            should_stop,
        } = *request;
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
        for (i, (stmt, diagnostic, unparsable_stmt)) in
            doc.iter(AnalyserDiagnosticsMapper).enumerate()
        {
            if (i + 1).is_multiple_of(STOP_CHECK_INTERVAL) && should_stop() {
                return None;
            }
            analysable_stmts.extend(stmt.filter(|stmt| !is_skipped(stmt.range)));
            syntax_diagnostics.extend(diagnostic);
            unparsable.extend(unparsable_stmt.filter(|stmt| !is_skipped(stmt.range)));
        }

        let result = analyser.analyse_until(
            AnalyserParams {
                stmts: analysable_stmts,
                unparsable,
                catalog_base,
                search_path,
                file_kind: self.file_kind(settings, path.as_path()),
                sections,
                typecheck,
            },
            should_stop,
        );
        if result.stopped {
            return None;
        }

        Some(DocumentAnalysis {
            result,
            syntax_diagnostics,
            disabled_rules,
            skipped_sections,
        })
    }

    /// The lint diagnostics of a document that are not suppressed.
    pub(super) fn lint_diagnostics(
        &self,
        project_key: ProjectKey,
        settings: &Settings,
        path: &PgLSPath,
        doc: &Document,
        only: &[RuleSelector],
        skip: &[RuleSelector],
    ) -> Vec<LinterDiagnostic> {
        // `fix_file` analyses documents of its own, which nobody else changes.
        let Some(analysis) = self.analyse_document(
            project_key,
            settings,
            path,
            doc,
            AnalysisRequest::all(only, skip, &|| false),
        ) else {
            return Vec::new();
        };
        let suppressions = doc.suppressions();
        analysis
            .lint_diagnostics()
            .iter()
            .filter(|diagnostic| !suppressions.is_suppressed(*diagnostic))
            .cloned()
            .collect()
    }
}
