use std::num::NonZeroU64;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use pgls_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_lexer::SyntaxKind;
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::OffsetMap};
use pgls_statement_splitter::{SplitResult, TextChange};
use pgls_suppressions::Suppressions;
use pgls_text_size::{TextRange, TextSize};
use pgls_walk::{AnalysableStatement, UnparsableStatement};
use rustc_hash::FxHashSet;

use pgls_migrations::{MigrationTool, Preprocessed, PsqlSyntaxDiagnostic, Script, Unit};
use pgls_templating::Template;

use crate::diagnostics::FileTooLarge;
use crate::workspace::ProjectKey;

use super::{
    document_cache::{ContentChange, DocumentCache, DocumentCacheStats, ParseResult, StaleEntries},
    statement_identifier::StatementId,
};

/// An open document. Clones share the cache, which is keyed so that each clone only sees what
/// was cached for its own content.
#[derive(Clone)]
pub(crate) struct Document {
    content: String,
    /// Changes on every change of the content or of the migration tool, see
    /// [`Document::generation`].
    generation: u64,
    version: i32,
    /// The statements of `content`, kept to split the next version of it incrementally.
    split: SplitResult,
    diagnostics: Vec<SDiagnostic>,
    cache: Arc<DocumentCache>,
    suppressions: Suppressions,
    /// The project whose settings `migration_tool` is from.
    project_key: ProjectKey,
    /// The version of the settings that `migration_tool` is from, see
    /// [`Document::has_migration_tool_of`].
    settings_version: u64,
    /// The migration tool that runs the file, from its settings. See [`Document::open`].
    migration_tool: MigrationTool,
    /// The format strings of a host language that the file is, from its settings.
    template: Template,
    /// How `migration_tool` runs the content, with the placeholders of `template`, see
    /// [`Document::script`].
    migration_script: Script,
    /// Where the last change of the content starts.
    edited_at: Option<TextSize>,
    /// `files.maxSize` of the settings that `migration_tool` is from.
    max_size: NonZeroU64,
    /// Whether the content is larger than `max_size`. Then it is neither split nor parsed: the
    /// document has no statements, and its only diagnostic is [`FileTooLarge`].
    too_large: bool,
}

/// A generation that no document had before, see [`Document::generation`].
fn next_generation() -> u64 {
    static GENERATIONS: AtomicU64 = AtomicU64::new(0);
    GENERATIONS.fetch_add(1, Ordering::Relaxed)
}

impl Document {
    /// A document without a project, and so without a migration tool: it is one migration.
    #[cfg(test)]
    pub(crate) fn new(content: String, version: i32) -> Document {
        Self::open(
            ProjectKey::default(),
            0,
            MigrationTool::Plain,
            Template::None,
            NonZeroU64::MAX,
            content,
            version,
        )
    }

    /// A document of the project `project_key`, whose settings at `settings_version` make
    /// `migration_tool` run the file, make it a `template`, and limit its size to `max_size`.
    /// When the settings or the project change, [`Document::set_settings`] applies them.
    pub(crate) fn open(
        project_key: ProjectKey,
        settings_version: u64,
        migration_tool: MigrationTool,
        template: Template,
        max_size: NonZeroU64,
        content: String,
        version: i32,
    ) -> Document {
        let generation = next_generation();
        let mut document = Document {
            split: pgls_statement_splitter::split(""),
            diagnostics: Vec::new(),
            content,
            generation,
            version,
            cache: Arc::new(DocumentCache::new(generation)),
            suppressions: Suppressions::default(),
            project_key,
            settings_version,
            migration_tool,
            template,
            migration_script: Script::default(),
            edited_at: None,
            max_size,
            too_large: false,
        };
        document.read();
        document
    }

    /// Splits the whole content, unless it is larger than `max_size`. Returns the previous
    /// migration script and statements.
    fn read(&mut self) -> (Script, SplitResult) {
        self.too_large = self.exceeds_max_size(&self.content);
        let (migration_script, split) = if self.too_large {
            let size = self.content.len();
            let limit = usize::try_from(self.max_size.get()).unwrap_or(usize::MAX);
            self.diagnostics = vec![SDiagnostic::new(FileTooLarge { size, limit })];
            self.suppressions = Suppressions::default();
            (Script::default(), pgls_statement_splitter::split(""))
        } else {
            let migration_script = self.script(&self.content);
            let split = pgls_statement_splitter::split(text(
                &self.content,
                migration_script.preprocessed.as_ref(),
            ));
            self.diagnostics = split_diagnostics(&split);
            self.suppressions =
                Suppressions::from(self.content.as_str()).with_statements(split.ranges.clone());
            (migration_script, split)
        };
        (
            std::mem::replace(&mut self.migration_script, migration_script),
            std::mem::replace(&mut self.split, split),
        )
    }

    /// How the migration tool runs `content`. A template's placeholders take the place of the
    /// tool's, like those of goose's `ENVSUB`: the host fills in the template before the tool
    /// sees it.
    fn script(&self, content: &str) -> Script {
        let mut script = self.migration_tool.read(content);
        if let Some(preprocessed) = self.template.read(content) {
            script.preprocessed = Some(preprocessed);
        }
        script
    }

    fn exceeds_max_size(&self, content: &str) -> bool {
        u64::try_from(content.len()).unwrap_or(u64::MAX) > self.max_size.get()
    }

    /// Splits the whole content again after it changed from `old_content` or the settings
    /// changed, unlike [`Document::update_content`], which splits only around the change.
    /// Returns what is no longer current in the cache, for the caller to drop.
    fn reread(&mut self, old_content: &str) -> StaleEntries {
        let (old_script, old_split) = self.read();
        self.generation = next_generation();
        self.edited_at = Some(TextSize::from(0));
        let stale = self.cache.content_changed(
            ContentChange {
                unchanged: TextSize::from(0),
                first_changed_statement: TextSize::from(0),
            },
            self.generation,
        );
        let old_text = text(old_content, old_script.preprocessed.as_ref());
        let removed = statements_not_in(
            old_split.ranges.iter().map(|range| &old_text[*range]),
            self.text(),
            &self.split.ranges,
        );
        self.cache.remove_statements(removed);
        stale
    }

    /// Replaces the content of the document. Returns what the change invalidated in its cache,
    /// for the caller to drop, see [`StaleEntries::drop_in_background`].
    pub(crate) fn update_content(&mut self, content: String, version: i32) -> StaleEntries {
        // A content larger than `max_size` has no statements to split the other version from.
        if self.too_large || self.exceeds_max_size(&content) {
            let old_content = std::mem::replace(&mut self.content, content);
            self.version = version;
            return self.reread(&old_content);
        }
        let migration_script = self.script(&content);
        // The text that is split is the content with the placeholders of the migration tool.
        let old_text = self.text();
        let new_text = text(&content, migration_script.preprocessed.as_ref());
        let change = TextChange::between(old_text, new_text);
        let unchanged = TextSize::try_from(change.start).unwrap_or_default();
        // Only the statements around the change are split again.
        let split = pgls_statement_splitter::split_edited(old_text, &self.split, new_text);
        let changed = changed_statements(
            &self.split.ranges,
            &split.ranges,
            unchanged,
            TextSize::try_from(change.old_end).unwrap_or_default(),
            new_text.len() as i64 - old_text.len() as i64,
        );
        // The statements before the change keep what the walks cached for them. The splitter
        // looks ahead, so a statement before the change can still end elsewhere.
        let first_changed_statement = self
            .split
            .ranges
            .get(changed.start)
            .map_or(TextSize::from(u32::MAX), |range| range.start());
        let old_content = std::mem::replace(&mut self.content, content);
        let old_script = std::mem::replace(&mut self.migration_script, migration_script);
        let old_split = std::mem::replace(&mut self.split, split);
        self.version = version;
        self.generation = next_generation();
        self.edited_at = Some(unchanged);

        let ranges = &self.split.ranges;
        self.suppressions =
            Suppressions::from(self.content.as_str()).with_statements(ranges.clone());
        self.diagnostics = split_diagnostics(&self.split);
        let stale = self.cache.content_changed(
            ContentChange {
                unchanged,
                first_changed_statement,
            },
            self.generation,
        );
        // The statements around the change are new to the checks against the database, which
        // only run on save, see `DatabaseResults`.
        #[cfg(feature = "db")]
        {
            let after = old_split.ranges.len().saturating_sub(changed.end);
            let added = changed.start..self.split.ranges.len().saturating_sub(after);
            let text = self.text();
            self.cache.statements_edited(
                self.split
                    .ranges
                    .get(added)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|range| text.get(std::ops::Range::<usize>::from(*range))),
            );
        }
        // The statements around the change that are no longer in the document drop what was
        // cached for them, with the statements of their function bodies. The other statements
        // have the same text as before.
        let old_text = text(&old_content, old_script.preprocessed.as_ref());
        let removed = statements_not_in(
            old_split
                .ranges
                .get(changed)
                .unwrap_or_default()
                .iter()
                .map(|range| &old_text[*range]),
            self.text(),
            &self.split.ranges,
        );
        self.cache.remove_statements(removed);
        stale
    }

    /// The client's version of the content.
    pub(crate) fn version(&self) -> i32 {
        self.version
    }

    /// Which version of the content this is. Unlike the client's version, it is unique across
    /// all documents: it changes on every change, also of the migration tool, and a document that
    /// is closed and opened again gets a new one.
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// Where the last change of the content starts.
    pub(crate) fn edited_at(&self) -> Option<TextSize> {
        self.edited_at
    }

    /// What is cached for the document.
    pub(crate) fn cache(&self) -> &DocumentCache {
        &self.cache
    }

    /// Applies the migration tool, the template syntax and the size limit of the settings of
    /// `project_key` at `settings_version`. Returns what the change invalidated in the cache, for
    /// the caller to drop, see [`StaleEntries::drop_in_background`].
    pub(crate) fn set_settings(
        &mut self,
        project_key: ProjectKey,
        settings_version: u64,
        migration_tool: MigrationTool,
        template: Template,
        max_size: NonZeroU64,
    ) -> StaleEntries {
        self.project_key = project_key;
        self.settings_version = settings_version;
        self.migration_tool = migration_tool;
        self.template = template;
        let max_size_changed = max_size != self.max_size;
        self.max_size = max_size;
        if self.too_large || (max_size_changed && self.exceeds_max_size(&self.content)) {
            // Settings change rarely, so the copy is cheaper than keeping the statements' texts
            // apart from the content.
            let content = self.content.clone();
            return self.reread(&content);
        }
        let migration_script = self.script(&self.content);
        let resplit = migration_script.preprocessed != self.migration_script.preprocessed;
        let old_script = std::mem::replace(&mut self.migration_script, migration_script);
        let old_split = resplit.then(|| {
            let split = pgls_statement_splitter::split(self.text());
            self.diagnostics = split_diagnostics(&split);
            self.suppressions =
                Suppressions::from(self.content.as_str()).with_statements(split.ranges.clone());
            std::mem::replace(&mut self.split, split)
        });
        // The analysis and the walks depend on the sections. They may have been cached for the
        // new settings before they reached the document, so the document gets a new generation:
        // requests that still run with the old tool are outdated and don't cache what they
        // compute. The parse results only depend on the text, and stay.
        self.generation = next_generation();
        let stale = self.cache.content_changed(
            ContentChange {
                unchanged: TextSize::from(0),
                first_changed_statement: TextSize::from(0),
            },
            self.generation,
        );
        // With other placeholders, the statements that are no longer in the text drop what was
        // cached for them.
        if let Some(old_split) = old_split {
            let old_text = text(&self.content, old_script.preprocessed.as_ref());
            let removed = statements_not_in(
                old_split.ranges.iter().map(|range| &old_text[*range]),
                self.text(),
                &self.split.ranges,
            );
            self.cache.remove_statements(removed);
        }
        stale
    }

    /// Whether the migration tool is the one of the settings of `project_key` at
    /// `settings_version`. Otherwise the settings changed since, or the file now belongs to
    /// another project, e.g. after the workspace folders changed.
    pub(crate) fn has_migration_tool_of(
        &self,
        project_key: ProjectKey,
        settings_version: u64,
    ) -> bool {
        self.project_key == project_key && self.settings_version == settings_version
    }

    pub(crate) fn project_key(&self) -> ProjectKey {
        self.project_key
    }

    pub(crate) fn settings_version(&self) -> u64 {
        self.settings_version
    }

    /// The migration tool that runs the file, from its settings.
    pub(crate) fn migration_tool(&self) -> MigrationTool {
        self.migration_tool
    }

    /// The format strings of a host language that the file is, from its settings.
    pub(crate) fn template(&self) -> Template {
        self.template
    }

    /// `files.maxSize` of the settings of the file.
    pub(crate) fn max_size(&self) -> NonZeroU64 {
        self.max_size
    }

    /// How the migration tool runs the document.
    pub(crate) fn migration_script(&self) -> &Script {
        &self.migration_script
    }

    /// The migration sections of the document, like the up and down migrations of dbmate and
    /// goose. A tool without section markers has none.
    pub(crate) fn migration_sections(&self) -> &[Unit] {
        &self.migration_script.units
    }

    /// What the caches of the document hold, for tests of their memory. It doesn't fill them.
    pub(crate) fn cache_stats(&self) -> DocumentCacheStats {
        let statements: FxHashSet<StatementId> = self
            .split
            .ranges
            .iter()
            .map(|range| StatementId::new(&self.text()[*range]))
            .collect();
        self.cache.stats(&statements)
    }

    pub(crate) fn suppressions(&self) -> &Suppressions {
        &self.suppressions
    }

    /// The ranges of the top-level statements, in file order.
    pub(crate) fn statement_ranges(&self) -> &[TextRange] {
        &self.split.ranges
    }

    /// The text and the range of each top-level statement, in file order.
    #[cfg(feature = "db")]
    pub(crate) fn top_level_statements(&self) -> impl Iterator<Item = (&str, TextRange)> {
        let text = self.text();
        self.split
            .ranges
            .iter()
            .filter_map(|range| Some((text.get(std::ops::Range::<usize>::from(*range))?, *range)))
    }

    pub(crate) fn get_document_content(&self) -> &str {
        &self.content
    }

    /// The psql syntax in the document that its migration tool can't run, see
    /// [`MigrationTool::psql_diagnostics`]. None in a document too large to analyse.
    pub(crate) fn psql_diagnostics(&self) -> Vec<PsqlSyntaxDiagnostic> {
        if self.too_large {
            return Vec::new();
        }
        self.migration_tool.psql_diagnostics(self.text())
    }

    /// The text that is split and parsed: the content, with the placeholders of the migration
    /// tool if it replaces parts of it. It has the length of the content.
    pub(crate) fn text(&self) -> &str {
        text(&self.content, self.migration_script.preprocessed.as_ref())
    }

    /// Whether `range` overlaps a part of the content that the host replaces, like a goose
    /// `ENVSUB` expansion or a template directive. Edits there would write the placeholder into
    /// the file.
    pub(crate) fn touches_placeholder(&self, range: TextRange) -> bool {
        self.migration_script
            .preprocessed
            .as_ref()
            .is_some_and(|preprocessed| preprocessed.touches_placeholder(range))
    }

    /// What the host replaces in the content, if anything, see [`Document::text`].
    pub(crate) fn preprocessed(&self) -> Option<&Preprocessed> {
        self.migration_script.preprocessed.as_ref()
    }

    /// The first template placeholder in `range` that can't be read where it is, see
    /// [`pgls_templating::Ambiguity`].
    fn ambiguity_in(&self, range: TextRange) -> Option<&pgls_templating::Ambiguity> {
        let ambiguous = &self.migration_script.preprocessed.as_ref()?.ambiguous;
        let first =
            ambiguous.partition_point(|ambiguity| ambiguity.placeholder.start() < range.start());
        ambiguous
            .get(first)
            .filter(|ambiguity| ambiguity.placeholder.end() <= range.end())
    }

    /// Whether a psql meta-command such as `\gexec` or `\gset` ends the statement in `range`
    /// instead of a `;`. psql sends the query buffer at a `;` and again at a `\g` command, so a
    /// `;` added before one runs the statement twice.
    ///
    /// Cost: lexes the text between the statement and the next one.
    pub(crate) fn ends_at_meta_command(&self, range: TextRange) -> bool {
        let content = self.content.as_str();
        if content[range].trim_end().ends_with(';') {
            return false;
        }
        let next = self
            .split
            .ranges
            .partition_point(|other| other.start() < range.end());
        let gap_end = self
            .split
            .ranges
            .get(next)
            .map_or(content.len(), |other| usize::from(other.start()));
        let Some(gap) = content.get(usize::from(range.end())..gap_end) else {
            return false;
        };
        pgls_lexer::lex(gap)
            .tokens()
            .find(|kind| !kind.is_trivia())
            .is_some_and(|kind| kind == SyntaxKind::META_COMMAND)
    }

    /// Whether `range` overlaps a value that the migration tool replaces, but that the text
    /// holds as written, like a goose `ENVSUB` expansion in a string literal.
    fn touches_unknown_value(&self, range: TextRange) -> bool {
        self.migration_script
            .preprocessed
            .as_ref()
            .is_some_and(|preprocessed| preprocessed.touches_unknown_value(range))
    }

    pub(crate) fn document_diagnostics(&self) -> &Vec<SDiagnostic> {
        &self.diagnostics
    }

    /// Every statement of the document, in file order: each top-level statement, followed by
    /// the statements embedded in it, like the statements of a SQL function body.
    ///
    /// Cost: finding the embedded statements parses each top-level statement on a cold cache.
    pub(crate) fn statements(&self) -> impl Iterator<Item = DocumentStatement> + '_ {
        self.statements_in(&self.split.ranges)
    }

    /// The statements from the first top-level statement that starts at or after `offset` on.
    pub(crate) fn statements_from(
        &self,
        offset: TextSize,
    ) -> impl Iterator<Item = DocumentStatement> + '_ {
        let first = self
            .split
            .ranges
            .partition_point(|range| range.start() < offset);
        self.statements_in(self.split.ranges.get(first..).unwrap_or_default())
    }

    /// The statements that contain `position`, outermost first: the top-level statement, and the
    /// body of a SQL function if `position` is in it. Only the top-level statement at `position`
    /// is looked at.
    pub(crate) fn statements_at(
        &self,
        position: TextSize,
    ) -> impl Iterator<Item = DocumentStatement> + '_ {
        // The ranges are ordered and don't overlap, so only the first one that ends after
        // `position` can contain it.
        let first = self
            .split
            .ranges
            .partition_point(|range| range.end() <= position);
        self.statements_in(self.split.ranges.get(first..=first).unwrap_or_default())
            .filter(move |stmt| stmt.range.contains(position))
    }

    /// The statements that a completion at `position` can be in, see
    /// [`DocumentStatement::completes_at`]. Only the top-level statements near `position` are
    /// looked at.
    pub(crate) fn statements_near(
        &self,
        position: TextSize,
    ) -> impl Iterator<Item = DocumentStatement> + '_ {
        // A completion can be up to two characters after the end of a statement. The bodies of
        // SQL functions are inside their top-level statement.
        let ranges = &self.split.ranges;
        let first = ranges.partition_point(|range| range.end() + TextSize::from(2) <= position);
        let near = ranges.get(first..).unwrap_or_default();
        let near = near
            .get(..near.partition_point(|range| range.start() <= position))
            .unwrap_or_default();
        self.statements_in(near)
            .filter(move |stmt| stmt.completes_at(position))
    }

    /// The first statement with the id `id`. Only the top-level statement that it is or that it
    /// is embedded in is parsed.
    ///
    /// Cost: a comparison of the text of each top-level statement of the same length as the
    /// id's top-level statement.
    #[cfg(feature = "db")]
    pub(crate) fn statement(&self, id: &StatementId) -> Option<DocumentStatement> {
        let root = id.parent_content().unwrap_or(id.content());
        let text = self.text();
        let range = *self
            .split
            .ranges
            .iter()
            .find(|range| usize::from(range.len()) == root.len() && &text[**range] == root)?;
        self.statements_in(std::slice::from_ref(&range))
            .find(|stmt| stmt.id == *id)
    }

    /// The statements of the top-level statements at `ranges`, see [`Self::statements`].
    fn statements_in<'a>(
        &'a self,
        ranges: &'a [TextRange],
    ) -> impl Iterator<Item = DocumentStatement> + 'a {
        ranges.iter().flat_map(|range| {
            let root_id = StatementId::new(&self.text()[*range]);
            let embedded: Vec<_> = self
                .cache
                .embedded(&root_id, self.generation)
                .iter()
                .map(|embedded| DocumentStatement {
                    id: root_id.create_child(&embedded.sql),
                    range: embedded.range + range.start(),
                    offsets: embedded.offsets.clone(),
                })
                .collect();
            std::iter::once(DocumentStatement::top_level(root_id, *range)).chain(embedded)
        })
    }

    /// The text of `stmt`. For an embedded statement, it can differ from the source text, like
    /// a function body with escaped quotes, see [`DocumentStatement::offsets`].
    pub(crate) fn sql<'a>(&'a self, stmt: &'a DocumentStatement) -> &'a str {
        stmt.id.content()
    }

    /// The tree of `stmt`, or why it doesn't parse.
    pub(crate) fn parse(&self, stmt: &DocumentStatement) -> Arc<ParseResult> {
        self.cache.parse(&stmt.id, self.generation)
    }

    /// The tree-sitter tree of `stmt`.
    pub(crate) fn tree(&self, stmt: &DocumentStatement) -> Arc<tree_sitter::Tree> {
        self.cache.tree(&stmt.id, self.generation)
    }

    /// What the walk takes of `stmt`: the statement if it parses, its syntax error, and the
    /// statement again if it is a top-level statement that doesn't parse.
    pub(crate) fn walk_input(&self, stmt: DocumentStatement) -> StatementWalkInput {
        let DocumentStatement { id, range, offsets } = stmt;
        let maybe_node = self.cache.parse(&id, self.generation);

        let (ast_option, syntax_error) = match &*maybe_node {
            Ok(node) => {
                let plpgsql_result = self.cache.plpgsql_parse(&id, self.generation);
                if let Some(Err(diag)) = plpgsql_result {
                    // offset the pgpsql diagnostic from the parent statement start
                    let span = diag.location().span.map(|sp| sp + range.start());
                    (Some(node.clone()), Some(diag.span(span.unwrap_or(range))))
                } else {
                    (Some(node.clone()), None)
                }
            }
            Err(diag) => (None, Some(diag.clone().span(range))),
        };
        // A template placeholder that can't be read where it is makes the statement a syntax
        // error that says why, also where the text with its placeholder happens to parse.
        let (ast_option, syntax_error) = match self.ambiguity_in(range) {
            Some(ambiguity) => (
                None,
                Some(SyntaxDiagnostic::new(
                    ambiguity.message(&self.content),
                    Some(ambiguity.placeholder),
                )),
            ),
            None => (ast_option, syntax_error),
        };
        let unparsable =
            (ast_option.is_none() && id.parent().is_none()).then(|| UnparsableStatement {
                range,
                sql: id.content().to_owned(),
            });

        // Statements in the body of a SQL function can reference its parameters.
        let function = id.parent().and_then(|root| {
            let parent = self.cache.parse(&root, self.generation);
            let Ok(pgls_query::NodeEnum::CreateFunctionStmt(create_fn)) =
                parent.as_ref().as_deref()
            else {
                return None;
            };
            pgls_resolve::FunctionContext::from_create_function(create_fn)
        });

        StatementWalkInput {
            analysable: ast_option.map(|root| {
                let conversion =
                    pgls_lexer::convert_to_positional_params_with_metadata(id.content());
                AnalysableStatement {
                    offsets,
                    ..AnalysableStatement::new(root, range).with_sql(id.content())
                }
                .with_function(function)
                .with_identifier_parameters(conversion.has_identifier_parameters)
                // psql fills in the string of a `:'name'` when it runs the statement.
                // Two arguments of a template that got the same placeholder are one parameter.
                .with_unknown_values(
                    self.touches_unknown_value(range)
                        || !conversion.string_replacements.is_empty()
                        || self
                            .preprocessed()
                            .is_some_and(|preprocessed| preprocessed.shares_names_in(range)),
                )
            }),
            syntax_error,
            unparsable,
        }
    }
}

/// What the walk takes of a statement, see [`Document::walk_input`].
pub(crate) struct StatementWalkInput {
    /// The statement, if it parses.
    pub analysable: Option<AnalysableStatement>,
    /// Why the statement doesn't parse, or why the PL/pgSQL body of the function it creates
    /// doesn't.
    pub syntax_error: Option<SyntaxDiagnostic>,
    /// The statement again if it is a top-level statement that doesn't parse.
    pub unparsable: Option<UnparsableStatement>,
}

/// A statement of a document: a top-level statement, or a statement embedded in one, like a
/// statement of a SQL function body. Its text, tree and parse result are read from the
/// [`Document`].
#[derive(Debug, Clone)]
pub(crate) struct DocumentStatement {
    id: StatementId,
    /// Where the statement is in the document.
    range: TextRange,
    /// Maps offsets in the statement text, `id.content()`, to offsets in the document, relative
    /// to the start of `range`. They only differ for embedded statements whose text isn't the
    /// source text, like a function body with escaped quotes.
    offsets: OffsetMap,
}

impl DocumentStatement {
    fn top_level(id: StatementId, range: TextRange) -> Self {
        Self {
            id,
            range,
            offsets: OffsetMap::default(),
        }
    }

    /// The id of the statement, which clients pass back to execute it.
    pub(crate) fn id(&self) -> &StatementId {
        &self.id
    }

    /// Where the statement is in the document.
    pub(crate) fn range(&self) -> TextRange {
        self.range
    }

    /// Maps offsets in the statement text to offsets in the document, relative to the start of
    /// [`Self::range`]. They only differ for embedded statements whose text isn't the source
    /// text, like a function body with escaped quotes.
    pub(crate) fn offsets(&self) -> &OffsetMap {
        &self.offsets
    }

    /// Whether the statement is embedded in a top-level statement, like a statement of a SQL
    /// function body.
    pub(crate) fn is_embedded(&self) -> bool {
        self.id.is_child()
    }

    /// Whether the statement is embedded in the top-level statement `other`.
    pub(crate) fn is_embedded_in(&self, other: &DocumentStatement) -> bool {
        self.id.is_child_of(&other.id)
    }

    /// The range in the document of `range` in the statement text. Like [`Self::statement_offset`],
    /// it can end after the statement text.
    pub(crate) fn document_range(&self, range: TextRange) -> TextRange {
        self.offsets.range_to_source(range) + self.range.start()
    }

    /// The offset in the statement text of `offset` in the document, which is in the
    /// statement's range, or up to two bytes after the end of an unterminated statement, where
    /// [`Self::completes_at`] finds the cursor of a completion.
    pub(crate) fn statement_offset(&self, offset: TextSize) -> TextSize {
        debug_assert!(self.range.start() <= offset);
        self.offsets
            .to_statement(offset.checked_sub(self.range.start()).unwrap_or_default())
    }

    /// Whether a completion at `position` can be in the statement. We allow an offset of two
    /// for the statement:
    ///
    /// ```sql
    /// select * from | <-- we want to suggest items for the next token.
    /// ```
    ///
    /// However, if the statement is terminated by a semicolon, we don't apply any offset:
    ///
    /// ```sql
    /// select * from users; | <-- no autocompletions here.
    /// ```
    fn completes_at(&self, position: TextSize) -> bool {
        let range = self.range;
        let measuring_range = if self.id.content().ends_with(';') {
            range
        } else {
            range.checked_expand_end(2.into()).unwrap_or(range)
        };
        measuring_range.contains(position)
    }
}

/// See [`Document::text`].
fn text<'a>(content: &'a str, preprocessed: Option<&'a Preprocessed>) -> &'a str {
    preprocessed.map_or(content, |preprocessed| &preprocessed.text)
}

/// The indexes of the statements of the split `old` that an edit changed, given the split `new`
/// of the text after it. The text before `unchanged`, and from `unchanged_from` on in the old
/// text, is the same, and `shift` is the change in length. A statement is unchanged if it is in
/// the same text, and the same range there, as before. Only the statements around the change
/// are compared.
fn changed_statements(
    old: &[TextRange],
    new: &[TextRange],
    unchanged: TextSize,
    unchanged_from: TextSize,
    shift: i64,
) -> std::ops::Range<usize> {
    let before = old
        .iter()
        .zip(new)
        .take_while(|(old, new)| old == new && old.end() <= unchanged)
        .count();
    let shifted = |range: &TextRange| {
        let start = u32::try_from(i64::from(u32::from(range.start())) + shift).ok()?;
        Some(TextRange::at(start.into(), range.len()))
    };
    let after = old
        .get(before..)
        .unwrap_or_default()
        .iter()
        .rev()
        .zip(new.get(before..).unwrap_or_default().iter().rev())
        .take_while(|(old, new)| old.start() >= unchanged_from && shifted(old) == Some(**new))
        .count();
    before..old.len() - after
}

/// The texts of `statements` that aren't the text of one of the statements at `ranges` of
/// `content`. Only the statements of the same length as one of `statements` are compared.
fn statements_not_in<'a>(
    statements: impl Iterator<Item = &'a str>,
    content: &str,
    ranges: &[TextRange],
) -> Vec<&'a str> {
    let mut missing: FxHashSet<&str> = statements.collect();
    if missing.is_empty() {
        return Vec::new();
    }
    let lengths: FxHashSet<usize> = missing.iter().map(|text| text.len()).collect();
    for range in ranges {
        if lengths.contains(&usize::from(range.len())) {
            missing.remove(&content[*range]);
            if missing.is_empty() {
                break;
            }
        }
    }
    missing.into_iter().collect()
}

/// The errors of the statement splitter as diagnostics.
fn split_diagnostics(split: &SplitResult) -> Vec<SDiagnostic> {
    split
        .errors
        .iter()
        .map(|err| SDiagnostic::new(err.clone().with_file_span(err.location().span)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_line_index::{LineIndex, PositionEncoding, WideEncoding, WideLineCol};
    use proptest::prelude::*;

    impl Document {
        /// The walk input of every statement.
        fn walk_inputs(&self) -> Vec<StatementWalkInput> {
            self.statements()
                .map(|stmt| self.walk_input(stmt))
                .collect()
        }

        /// Fails unless the document is the document opened on its content with its settings:
        /// the same text, statements, diagnostics, suppressions and migration script, the same
        /// embedded statements and parse results from the cache, and every range inside the text.
        fn equals_a_fresh_document(&self) -> Result<(), TestCaseError> {
            let fresh = Document::open(
                self.project_key,
                self.settings_version,
                self.migration_tool,
                self.template,
                self.max_size,
                self.content.clone(),
                self.version,
            );
            let text = self.text();
            prop_assert_eq!(text, fresh.text());
            prop_assert_eq!(self.too_large, fresh.too_large);
            prop_assert_eq!(&self.split, &fresh.split);
            prop_assert_eq!(
                format!("{:?}", self.diagnostics),
                format!("{:?}", fresh.diagnostics)
            );
            prop_assert_eq!(
                format!("{:?}", self.suppressions),
                format!("{:?}", fresh.suppressions)
            );
            prop_assert_eq!(
                format!("{:?}", self.migration_script),
                format!("{:?}", fresh.migration_script)
            );
            for diagnostic in &self.diagnostics {
                let span = diagnostic.location().span;
                prop_assert!(
                    span.is_none_or(|span| text
                        .get(std::ops::Range::<usize>::from(span))
                        .is_some()),
                    "{span:?} is not inside the text"
                );
            }

            let statements: Vec<_> = self.statements().collect();
            let fresh_statements: Vec<_> = fresh.statements().collect();
            prop_assert_eq!(format!("{statements:?}"), format!("{fresh_statements:?}"));
            for stmt in &statements {
                prop_assert!(
                    text.get(std::ops::Range::<usize>::from(stmt.range()))
                        .is_some(),
                    "{:?} is not inside the text",
                    stmt.range()
                );
                prop_assert_eq!(
                    format!("{:?}", self.parse(stmt)),
                    format!("{:?}", fresh.parse(stmt))
                );
            }
            Ok(())
        }
    }

    /// A change of the content as a client sends it: the text that replaces the content between
    /// two UTF-16 positions, which can be past the end of their line or of the content, or inside
    /// a character.
    #[derive(Debug, Clone)]
    struct ClientChange {
        start: WideLineCol,
        end: WideLineCol,
        text: String,
    }

    impl ClientChange {
        /// The statements that the texts are mostly made of, so that a change has statements on
        /// both sides that it doesn't touch. They have all three line breaks and characters of 2
        /// and 4 bytes in UTF-8 in and between them.
        const STATEMENTS: &[&str] = &[
            "select 1;\n",
            "select x from t where y = 'é😀';\r\n",
            "select é, 😀 from t\rwhere x = 1;\r",
            "create function f() returns int language sql as $$ select 1; select 2 $$;\n",
            "create function g() returns int language sql begin atomic select 1; end;\n",
            "create function h() returns int language sql as 'select 1; select ''é''';\n",
            "copy t from stdin;\n1\n\\.\n",
            "insert into t values (${A}, ${B:-x});\n",
            "select * from %s.{t} where id = %d and x = '%%{{' and y = 7 %% 2;\n",
            "update t set x = 1\n\nselect 2;\n",
        ];

        /// The pieces that a change can open or close a statement, a string, a comment or a
        /// function body with, line breaks, the markers of the migration tools and of
        /// suppressions, and the directives of the templates.
        const PIECES: &[&str] = &[
            "select",
            " ",
            "x",
            "é",
            "😀",
            "\n",
            "\r\n",
            "\r",
            ";",
            "'",
            "\"",
            "$$",
            "$f$",
            "(",
            ")",
            "--",
            "/*",
            "*/",
            "\\",
            "-- pgls-ignore lint\n",
            "-- +goose Up\n",
            "-- +goose Down\n",
            "-- +goose ENVSUB ON\n",
            "-- +goose ENVSUB OFF\n",
            "-- +goose StatementBegin\n",
            "-- +goose StatementEnd\n",
            "-- migrate:up\n",
            "-- migrate:down\n",
            "%",
            "%d",
            "{",
            "}",
            "{x}",
        ];

        /// Up to `parts_max` of [`Self::STATEMENTS`] and [`Self::PIECES`], `statement_weight`
        /// statements to one piece.
        fn text(parts_max: usize, statement_weight: u32) -> impl Strategy<Value = String> {
            proptest::collection::vec(
                prop_oneof![
                    statement_weight => proptest::sample::select(Self::STATEMENTS),
                    1 => proptest::sample::select(Self::PIECES),
                ],
                0..=parts_max,
            )
            .prop_map(|parts| parts.concat())
        }

        fn arbitrary() -> impl Strategy<Value = Self> {
            // Mostly typing: a change of a few characters within one line.
            let lines = prop_oneof![4 => Just(0), 1 => 1..3u32];
            (0..48u32, 0..48u32, lines, 0..16u32, Self::text(2, 1)).prop_map(
                |(line, col, lines, width, text)| ClientChange {
                    start: WideLineCol { line, col },
                    end: WideLineCol {
                        line: line + lines,
                        col: if lines == 0 { col + width } else { width },
                    },
                    text,
                },
            )
        }

        /// Applies the change to `content`, with the positions mapped to offsets like the server
        /// maps them: a position past the end of its line is the end of the line, and one inside
        /// a character is the start of the character.
        fn apply(&self, content: &mut String) {
            let index = LineIndex::new(content);
            let offset = |position| {
                usize::from(index.offset_at(position, PositionEncoding::Wide(WideEncoding::Utf16)))
            };
            let (start, end) = (offset(self.start), offset(self.end));
            content.replace_range(start.min(end)..start.max(end), &self.text);
        }
    }

    /// What happens to an open document: a notification of changes, which the server applies in
    /// order and passes on as one new version, or new settings.
    #[derive(Debug, Clone)]
    enum DocumentEvent {
        Changes(Vec<ClientChange>),
        Settings(MigrationTool, Template, NonZeroU64),
    }

    impl DocumentEvent {
        fn arbitrary() -> impl Strategy<Value = Self> {
            prop_oneof![
                9 => proptest::collection::vec(ClientChange::arbitrary(), 1..3)
                    .prop_map(DocumentEvent::Changes),
                1 => Self::settings().prop_map(|(tool, template, max_size)| {
                    DocumentEvent::Settings(tool, template, max_size)
                }),
            ]
        }

        /// A migration tool, a template syntax, and a size limit that the content is often above.
        fn settings() -> impl Strategy<Value = (MigrationTool, Template, NonZeroU64)> {
            (
                proptest::sample::select(vec![
                    MigrationTool::Plain,
                    MigrationTool::Supabase,
                    MigrationTool::Dbmate,
                    MigrationTool::Goose,
                ]),
                proptest::sample::select(vec![
                    Template::None,
                    Template::GoPrintf,
                    Template::RustFmt,
                ]),
                prop_oneof![
                    3 => Just(NonZeroU64::MAX),
                    1 => (0..256u64).prop_map(|size| NonZeroU64::MIN.saturating_add(size)),
                ],
            )
        }
    }

    proptest! {
        /// rust-analyzer's `reparse` fuzzing for documents: after random changes, like a client
        /// sends them, every version of the document equals the document opened on its content,
        /// although it only splits the text around a change and keeps its cache.
        #[test]
        fn edited_documents_equal_the_documents_opened_on_their_content(
            (tool, template, max_size) in DocumentEvent::settings(),
            content in ClientChange::text(40, 4),
            events in proptest::collection::vec(DocumentEvent::arbitrary(), 1..12),
        ) {
            let mut content = content;
            let mut d = Document::open(
                ProjectKey::default(),
                0,
                tool,
                template,
                max_size,
                content.clone(),
                0,
            );
            d.equals_a_fresh_document()?;
            for (version, event) in (1..).zip(events) {
                match event {
                    DocumentEvent::Changes(changes) => {
                        for change in &changes {
                            change.apply(&mut content);
                        }
                        d.update_content(content.clone(), version).drop_in_background();
                    }
                    DocumentEvent::Settings(tool, template, max_size) => {
                        d.set_settings(
                            ProjectKey::default(),
                            version.unsigned_abs().into(),
                            tool,
                            template,
                            max_size,
                        )
                        .drop_in_background();
                    }
                }
                prop_assert_eq!(d.get_document_content(), content.as_str());
                d.equals_a_fresh_document()?;
            }
        }
    }

    #[test]
    fn sql_function_body() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let d = Document::new(input.to_string(), 1);

        let stmts = d.statements().collect::<Vec<_>>();

        assert_eq!(stmts.len(), 2);
        assert_eq!(d.sql(&stmts[1]), "select $1 + $2;");
    }

    #[test]
    fn sql_function_body_sees_the_parameters() {
        let input =
            "CREATE FUNCTION add(a int, b int) RETURNS int AS 'SELECT a + $2;' LANGUAGE sql;";
        let d = Document::new(input.to_string(), 1);

        let results = d.walk_inputs();

        assert_eq!(results.len(), 2);
        assert!(results[0].analysable.as_ref().unwrap().function.is_none());
        let body = results[1].analysable.as_ref().unwrap();
        assert_eq!(body.sql.as_deref(), Some("SELECT a + $2;"));
        let function = body.function.as_ref().unwrap();
        assert_eq!(function.function_name, "add");
        assert_eq!(
            function
                .params
                .iter()
                .map(|param| param.name.as_deref())
                .collect::<Vec<_>>(),
            [Some("a"), Some("b")]
        );
    }

    #[test]
    fn sql_function_body_with_several_statements() {
        let input = "create function f(a int) returns int language sql as $$\n  insert into t values (a);\n  select a;\n$$;";
        let d = Document::new(input.to_string(), 1);

        let results = d.walk_inputs();

        assert_eq!(results.len(), 3);
        for result in &results[1..] {
            let syntax_error = &result.syntax_error;
            assert!(syntax_error.is_none(), "{syntax_error:?}");
            let statement = result.analysable.as_ref().unwrap();
            assert!(statement.function.is_some());
            assert_eq!(
                Some(&input[statement.range]),
                statement.sql.as_deref(),
                "the range is the statement's range in the document"
            );
        }
        assert_eq!(
            results[1].analysable.as_ref().unwrap().sql.as_deref(),
            Some("insert into t values (a);")
        );
        assert_eq!(
            results[2].analysable.as_ref().unwrap().sql.as_deref(),
            Some("select a;")
        );
    }

    #[test]
    fn return_bodies_have_no_syntax_error() {
        for input in [
            "create function f(p int) returns int begin atomic return p + 1; end;",
            "create function return(p int) returns int return p + 1;",
            "create function f(return int) returns int return return + 1;",
        ] {
            let d = Document::new(input.to_string(), 1);

            let results = d.walk_inputs();

            assert_eq!(results.len(), 2, "{input}");
            for result in &results {
                let syntax_error = &result.syntax_error;
                assert!(syntax_error.is_none(), "{input}: {syntax_error:?}");
            }
        }
    }

    #[test]
    fn empty_sql_function_body_has_no_syntax_error() {
        let input = "create function f() returns void language sql as $$ $$;";
        let d = Document::new(input.to_string(), 1);

        let results = d.walk_inputs();

        assert_eq!(results.len(), 1);
        assert!(results[0].syntax_error.is_none());
    }

    #[test]
    fn walk_input_has_the_plpgsql_syntax_error() {
        let input = "
CREATE FUNCTION test_func()
    RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- syntax error: missing semicolon and typo
    DECLAR x integer
    x := 10;
END;
$$;";

        let d = Document::new(input.to_string(), 1);
        let results = d.walk_inputs();

        assert_eq!(results.len(), 1);
        let StatementWalkInput {
            analysable: ast,
            syntax_error: diagnostic,
            ..
        } = &results[0];

        // Should have parsed the CREATE FUNCTION statement
        assert!(ast.is_some());

        // Should have a PL/pgSQL syntax error
        assert!(diagnostic.is_some());
        assert_eq!(
            format!("{:?}", diagnostic.as_ref().unwrap().message),
            "Invalid statement: syntax error at or near \"DECLAR\""
        );
    }

    #[test]
    fn walk_input_of_a_valid_plpgsql_function() {
        let input = "
CREATE FUNCTION valid_func()
    RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    x integer := 5;
BEGIN
    RETURN x * 2;
END;
$$;";

        let d = Document::new(input.to_string(), 1);
        let results = d.walk_inputs();

        assert_eq!(results.len(), 1);
        let StatementWalkInput {
            analysable: ast,
            syntax_error: diagnostic,
            ..
        } = &results[0];

        // Should have parsed the CREATE FUNCTION statement
        assert!(ast.is_some());

        // Should NOT have any PL/pgSQL syntax errors
        assert!(diagnostic.is_none());
    }

    #[test]
    fn walk_input_of_a_plpgsql_function_twice() {
        let input = "
CREATE FUNCTION cached_func()
    RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE NOTICE 'Testing cache';
END;
$$;";

        let d = Document::new(input.to_string(), 1);

        let results1 = d.walk_inputs();
        assert_eq!(results1.len(), 1);
        assert!(results1[0].analysable.is_some());
        assert!(results1[0].syntax_error.is_none());

        let results2 = d.walk_inputs();
        assert_eq!(results2.len(), 1);
        assert!(results2[0].analysable.is_some());
        assert!(results2[0].syntax_error.is_none());
    }

    #[test]
    fn statements_have_their_text_and_range() {
        let input = "SELECT 1; INSERT INTO users VALUES (1);";
        let d = Document::new(input.to_string(), 1);

        let results = d.statements().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        assert_eq!(d.sql(&results[0]), "SELECT 1;");
        assert_eq!(d.sql(&results[1]), "INSERT INTO users VALUES (1);");

        assert_eq!(results[0].range(), TextRange::new(0.into(), 9.into()));
        assert_eq!(results[1].range(), TextRange::new(10.into(), 39.into()));
    }

    #[test]
    fn hstore_delete_function_has_no_syntax_diagnostic() {
        let input = "SELECT delete(resource_attributes, 'gen_ai.system');";
        let d = Document::new(input.to_string(), 1);

        assert!(d.document_diagnostics().is_empty());

        let results = d.walk_inputs();
        assert_eq!(results.len(), 1);
        assert!(results[0].analysable.is_some());
        assert!(results[0].syntax_error.is_none());
    }

    #[test]
    fn statements_parse_or_have_a_syntax_error() {
        let input = "SELECT 1; INVALID SYNTAX HERE;";
        let d = Document::new(input.to_string(), 1);

        let results = d.statements().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        // First statement should parse successfully
        assert_eq!(d.sql(&results[0]), "SELECT 1;");
        assert!(d.parse(&results[0]).is_ok());

        // Second statement should fail to parse
        assert_eq!(d.sql(&results[1]), "INVALID SYNTAX HERE;");
        assert!(d.parse(&results[1]).is_err());
    }

    #[test]
    fn statements_have_a_tree() {
        let input = "
CREATE FUNCTION test_fn() RETURNS integer AS $$
BEGIN
    RETURN 42;
END;
$$ LANGUAGE plpgsql;";

        let d = Document::new(input.to_string(), 1);
        let results = d.statements().collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert!(d.parse(&results[0]).is_ok());
        assert_eq!(d.tree(&results[0]).root_node().kind(), "program");
    }

    #[test]
    fn statements_of_a_sql_function_body_parse() {
        let input =
            "CREATE FUNCTION add(a int, b int) RETURNS int AS 'SELECT $1 + $2;' LANGUAGE sql;";
        let d = Document::new(input.to_string(), 1);

        let results = d.statements().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        // Check the function body
        assert!(results[1].is_embedded());
        assert!(results[1].is_embedded_in(&results[0]));
        assert_eq!(d.sql(&results[1]), "SELECT $1 + $2;");
        assert!(d.parse(&results[1]).is_ok());
    }

    #[test]
    fn statements_near_a_completion() {
        let input = "SELECT * FROM users; INSERT INTO";
        let d = Document::new(input.to_string(), 1);

        // No completions after the semicolon of the first statement.
        assert_eq!(d.statements_near(20.into()).count(), 0);

        // Up to two characters after the end of an unterminated statement.
        let results = d.statements_near(32.into()).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(d.sql(&results[0]), "INSERT INTO");
    }

    #[test]
    fn statements_at_yields_the_statements_at_the_position() {
        let input =
            "select 1;\ncreate function f() returns int language sql as 'select 2;';\nselect 3;";
        let d = Document::new(input.to_string(), 1);
        let at = |needle: &str| {
            let position = TextSize::try_from(input.find(needle).unwrap()).unwrap();
            d.statements_at(position)
                .map(|stmt| d.sql(&stmt).to_owned())
                .collect::<Vec<_>>()
        };

        assert_eq!(at("1;"), ["select 1;"]);
        // The `CREATE FUNCTION`, then its body.
        assert_eq!(
            at("2;"),
            [
                "create function f() returns int language sql as 'select 2;';",
                "select 2;"
            ]
        );
        assert_eq!(
            at("returns"),
            ["create function f() returns int language sql as 'select 2;';"]
        );
        assert_eq!(at("select 3"), ["select 3;"]);
        // Between two statements, and past the end.
        assert!(at("\ncreate").is_empty());
        assert!(d.statements_at(TextSize::of(input)).next().is_none());
    }

    #[test]
    fn keeps_the_parsed_statements_that_are_still_in_the_document() {
        let mut d = Document::new("select 1;\nselect 2;".to_string(), 1);
        let first = StatementId::new("select 1;");
        let second = StatementId::new("select 2;");
        let first_ast = d.cache.parse(&first, d.generation);
        let second_ast = d.cache.parse(&second, d.generation);
        let first_tree = d.cache.tree(&first, d.generation);

        d.update_content("select 1;\nselect 3;".to_string(), 2)
            .drop_in_background();

        assert!(Arc::ptr_eq(
            &first_ast,
            &d.cache.parse(&first, d.generation)
        ));
        assert!(Arc::ptr_eq(
            &first_tree,
            &d.cache.tree(&first, d.generation)
        ));
        assert!(!Arc::ptr_eq(
            &second_ast,
            &d.cache.parse(&second, d.generation)
        ));
    }

    #[test]
    #[cfg(feature = "db")]
    fn statement_finds_the_statement_with_the_id() {
        let input =
            "SELECT 1; SELECT 2;\ncreate function f() returns int language sql as 'select 3;';";
        let d = Document::new(input.to_string(), 1);

        let all = d.statements().collect::<Vec<_>>();
        assert_eq!(all.len(), 4);
        for stmt in &all {
            let found = d.statement(stmt.id()).unwrap();
            assert_eq!(found.range(), stmt.range());
            assert_eq!(d.sql(&found), d.sql(stmt));
        }

        // Neither a statement that isn't in the document, nor a body of another function.
        assert!(d.statement(&StatementId::new("SELECT 3;")).is_none());
        assert!(
            d.statement(&StatementId::new("SELECT 1;").create_child("select 3;"))
                .is_none()
        );
    }

    /// Each change splits only the statements around it, with the same result as splitting the
    /// whole document.
    #[test]
    fn updates_split_like_the_whole_document() {
        let statements = [
            "create table t (id bigint primary key, name text);",
            "create function f(a int) returns int language sql as $$ select a $$;",
            "select c.relname from pg_class c where c.relnatts = 1;",
            "insert into t values (1, 'x');",
            "-- comment\nselect 1;",
            "update t set name = 'y'",
        ];
        let snippets = [
            "x",
            " ",
            "\n",
            "\n\n",
            ";",
            "select 2;",
            "'",
            "$$",
            "(",
            ")",
            "",
        ];
        let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
        let mut below = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let mut content: String = (0..200)
            .map(|i| statements[i % statements.len()])
            .collect::<Vec<_>>()
            .join("\n");
        let mut d = Document::new(content.clone(), 0);
        for version in 1..1_000 {
            let at = below(content.len() + 1);
            let end = (at + below(20)).min(content.len());
            let inserted = snippets[below(snippets.len())];
            content = format!("{}{inserted}{}", &content[..at], &content[end..]);
            d.update_content(content.clone(), version)
                .drop_in_background();

            let whole = pgls_statement_splitter::split(&content);
            assert_eq!(d.split, whole, "{content}");
            assert_eq!(
                format!("{:?}", d.diagnostics),
                format!("{:?}", split_diagnostics(&whole))
            );
        }
    }

    /// With goose's `ENVSUB`, the text that is split has placeholders, which can hide a `;`, and an
    /// edit can change it after the edit, like `ENVSUB ON`. Edits split it like the whole text is
    /// split.
    #[test]
    fn edits_split_the_text_with_the_placeholders_of_the_migration_tool() {
        let goose = MigrationTool::Goose;
        let lines = [
            "-- +goose Up",
            "-- +goose ENVSUB ON",
            "create schema ${SCHEMA:-app};",
            "select '${A}'; select ${B};",
            "-- +goose ENVSUB OFF",
            "select ${C};",
        ];
        let edits = [
            (1, "-- +goose ENVSUB OFF"),
            (1, "-- +goose ENVSUB ON"),
            (3, "select ${A; select ${B};"),
            (4, "-- +goose ENVSUB ON"),
            (2, "create schema ${SCHEMA:-app;x};"),
            (4, "-- +goose ENVSUB OFF"),
            (1, "-- +goose ENVSUB OFF"),
            (1, "-- +goose ENVSUB ON"),
        ];
        let mut lines = lines.map(str::to_string);
        let mut d = Document::open(
            ProjectKey::default(),
            0,
            goose,
            Template::None,
            NonZeroU64::MAX,
            lines.join("\n"),
            0,
        );
        for (version, (line, text)) in (1..).zip(edits) {
            lines[line] = text.to_string();
            let content = lines.join("\n");
            d.update_content(content.clone(), version)
                .drop_in_background();

            let whole = Document::open(
                ProjectKey::default(),
                0,
                goose,
                Template::None,
                NonZeroU64::MAX,
                content.clone(),
                0,
            );
            assert_eq!(d.split, whole.split, "{content}");
            assert_eq!(d.text(), whole.text(), "{content}");
        }
    }

    /// After every edit, the cache holds what it held for the statements that are still in the
    /// document, and nothing for the others: an edit drops the statements it removed, also when
    /// another statement has the same text, and the statements of their function bodies.
    #[test]
    fn edits_drop_only_the_statements_they_removed() {
        let statements = [
            "create table t (id bigint primary key, name text);",
            "create function f(a int) returns int language sql as $$ select a; select 2 $$;",
            "select 1;",
            "select 1;",
            "insert into t values (1, 'x');",
            "-- comment\nselect 2;",
        ];
        let snippets = ["x", " ", "\n\n", ";", "select 1;", "1", "(", ""];
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut below = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let mut content: String = (0..60)
            .map(|i| statements[i % statements.len()])
            .collect::<Vec<_>>()
            .join("\n");
        let mut d = Document::new(content.clone(), 0);
        let fill = |d: &Document| {
            for stmt in d.statements() {
                d.tree(&stmt);
                d.parse(&stmt);
            }
        };
        fill(&d);
        for version in 1..300 {
            let at = below(content.len() + 1);
            let end = (at + below(30)).min(content.len());
            let inserted = snippets[below(snippets.len())];
            content = format!("{}{inserted}{}", &content[..at], &content[end..]);
            d.update_content(content.clone(), version)
                .drop_in_background();
            // Nothing is cached for a statement that isn't in the document.
            let stats = d.cache_stats();
            assert!(stats.parse_results <= stats.statements, "{content}");
            assert!(stats.trees <= stats.statements, "{content}");
            // Every statement is cached once.
            fill(&d);
            let stats = d.cache_stats();
            assert_eq!(stats.parse_results, stats.statements, "{content}");
            assert_eq!(stats.trees, stats.statements, "{content}");
        }
    }

    #[test]
    fn finds_the_statements_an_edit_changed() {
        fn changed<'a>(old: &'a str, new: &str) -> Vec<&'a str> {
            let (old_split, new_split) = (
                pgls_statement_splitter::split(old),
                pgls_statement_splitter::split(new),
            );
            let change = TextChange::between(old, new);
            let range = changed_statements(
                &old_split.ranges,
                &new_split.ranges,
                TextSize::try_from(change.start).unwrap(),
                TextSize::try_from(change.old_end).unwrap(),
                new.len() as i64 - old.len() as i64,
            );
            old_split.ranges[range]
                .iter()
                .map(|range| &old[*range])
                .collect()
        }
        assert_eq!(
            changed(
                "select 1;\nselect 2;\nselect 3;",
                "select 1;\nselect 22;\nselect 3;"
            ),
            ["select 2;"]
        );
        // The same text as the statement before it.
        assert_eq!(
            changed(
                "select 1;\nselect 1;\nselect 1;",
                "select 1;\nselect 1;\n\nselect 1;"
            ),
            Vec::<&str>::new()
        );
        assert_eq!(
            changed("select 1;\nselect 2;", "select 1;\nselect 2;\nselect 3;"),
            Vec::<&str>::new()
        );
        // An edit across two statements changes both.
        assert_eq!(
            changed(
                "select 1;\nselect 2;\nselect 3;",
                "select 1;\nselect 4;\nselect 5;"
            ),
            ["select 2;", "select 3;"]
        );
        assert_eq!(
            changed("select 1;\nselect 2;\nselect 3;", "select 1;\nselect 2;\n"),
            ["select 3;"]
        );
        assert_eq!(changed("select 1;", ""), ["select 1;"]);
        // A change in a multi-byte character.
        assert_eq!(
            changed("select 'é';\nselect 2;", "select 'è';\nselect 2;"),
            ["select 'é';"]
        );
    }

    #[test]
    fn requests_on_a_previous_version_add_nothing_to_the_cache() {
        let previous = Document::new("select 1;\nselect 2;".into(), 1);
        let mut current = previous.clone();
        current
            .update_content("select 1;\nselect 3;".into(), 2)
            .drop_in_background();
        // `select 2;` is no longer in the document, so it stays out of the cache.
        for stmt in previous.statements() {
            previous.tree(&stmt);
            previous.parse(&stmt);
        }
        let stats = current.cache_stats();
        assert_eq!((stats.parse_results, stats.trees), (0, 0));
        // It still reads what is cached for the current version.
        let first = StatementId::new("select 1;");
        let cached = current.cache.parse(&first, current.generation);
        assert!(Arc::ptr_eq(
            &cached,
            &previous.cache.parse(&first, previous.generation)
        ));
    }

    /// A clone of a document shares its cache, but a request on the previous version neither
    /// reads nor writes what is cached for the new one.
    #[test]
    fn clones_only_see_the_cache_of_their_content() {
        use crate::workspace::{
            ProjectKey,
            server::document_cache::{CacheKey, MigrationContext, ResolvedStatement},
        };

        let previous = Document::new("select 1;".into(), 1);
        let mut current = previous.clone();
        current
            .update_content("select 2;".into(), 2)
            .drop_in_background();
        let key = CacheKey::new(0, ProjectKey::default(), None);
        let context = MigrationContext::default();
        let resolved = || -> Arc<[ResolvedStatement]> { Arc::from(Vec::new()) };

        previous
            .cache()
            .with_walk(&key, previous.generation(), &context, |walk| {
                walk.record_resolved(0.into(), resolved())
            });
        assert!(
            current
                .cache()
                .with_walk(&key, current.generation(), &context, |walk| walk
                    .resolved(0.into()))
                .is_none()
        );
        current
            .cache()
            .with_walk(&key, current.generation(), &context, |walk| {
                walk.record_resolved(0.into(), resolved())
            });
        assert!(
            previous
                .cache()
                .with_walk(&key, previous.generation(), &context, |walk| walk
                    .resolved(0.into()))
                .is_none()
        );
        assert!(
            current
                .cache()
                .with_walk(&key, current.generation(), &context, |walk| walk
                    .resolved(0.into()))
                .is_some()
        );
    }
}
