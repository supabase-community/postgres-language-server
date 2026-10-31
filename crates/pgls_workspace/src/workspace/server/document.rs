#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use pgls_analyser::{AnalysableStatement, UnparsableStatement};
use pgls_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, function_body::OffsetMap};
use pgls_statement_splitter::SplitResult;
use pgls_suppressions::Suppressions;
use pgls_text_size::{TextRange, TextSize};
use rustc_hash::FxHashSet;

use super::{
    document_cache::{ContentChange, DocumentCache, DocumentCacheStats, StaleEntries},
    statement_identifier::StatementId,
};

/// An open document. Clones share the cache, which is keyed so that each clone only sees what
/// was cached for its own content.
#[derive(Clone)]
pub struct Document {
    content: String,
    /// Changes on every change of the content, see [`Document::generation`].
    generation: u64,
    version: i32,
    /// The statements of `content`, kept to split the next version of it incrementally.
    split: SplitResult,
    diagnostics: Vec<SDiagnostic>,
    cache: Arc<DocumentCache>,
    suppressions: Suppressions,
    /// Where the last change of the content starts.
    edited_at: Option<TextSize>,
}

/// A generation that no document had before, see [`Document::generation`].
fn next_generation() -> u64 {
    static GENERATIONS: AtomicU64 = AtomicU64::new(0);
    GENERATIONS.fetch_add(1, Ordering::Relaxed)
}

impl Document {
    pub fn new(content: String, version: i32) -> Document {
        let split = pgls_statement_splitter::split(&content);
        let diagnostics = split_diagnostics(&split);
        let suppressions =
            Suppressions::from(content.as_str()).with_statements(split.ranges.clone());

        let generation = next_generation();
        Document {
            split,
            diagnostics,
            content,
            generation,
            version,
            cache: Arc::new(DocumentCache::new(generation)),
            suppressions,
            edited_at: None,
        }
    }

    /// Replaces the content of the document. Returns what the change invalidated in its cache,
    /// for the caller to drop, see [`StaleEntries::drop_in_background`].
    pub fn update_content(&mut self, content: String, version: i32) -> StaleEntries {
        let unchanged = common_prefix(&self.content, &content);
        // Only the statements around the change are split again.
        let split = pgls_statement_splitter::split_edited(&self.content, &self.split, &content);
        let ranges = &split.ranges;
        self.content = content;
        self.version = version;
        self.generation = next_generation();

        // The statements before the change keep what the walks cached for them. The splitter
        // looks ahead, so a statement before the change can still end elsewhere.
        let first_changed_statement = self
            .split
            .ranges
            .iter()
            .zip(ranges)
            .find(|(old, new)| old != new || old.end() > unchanged)
            .map_or_else(
                || {
                    self.split
                        .ranges
                        .get(ranges.len())
                        .map_or(TextSize::from(u32::MAX), |range| range.start())
                },
                |(old, _)| old.start(),
            );
        self.edited_at = Some(unchanged);

        self.suppressions =
            Suppressions::from(self.content.as_str()).with_statements(ranges.clone());
        self.diagnostics = split_diagnostics(&split);
        self.split = split;
        // Keep what is cached for the statements that are still in the document. The bodies
        // of SQL functions are kept with their function.
        let statements: FxHashSet<&str> = self
            .split
            .ranges
            .iter()
            .map(|range| &self.content[*range])
            .collect();
        let is_present =
            |id: &StatementId| statements.contains(id.parent_content().unwrap_or(id.content()));
        self.cache.retain_statements(is_present);
        self.cache.content_changed(
            ContentChange {
                unchanged,
                first_changed_statement,
            },
            self.generation,
        )
    }

    /// Which version of the content this is. Unlike the client's version, it is unique across
    /// all documents: it changes on every change, and a document that is closed and opened
    /// again gets a new one.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Where the last change of the content starts.
    pub fn edited_at(&self) -> Option<TextSize> {
        self.edited_at
    }

    /// What is cached for the document.
    pub(crate) fn cache(&self) -> &DocumentCache {
        &self.cache
    }

    /// What the caches of the document hold, for tests of their memory. It doesn't fill them.
    pub(crate) fn cache_stats(&self) -> DocumentCacheStats {
        let statements: FxHashSet<StatementId> = self
            .split
            .ranges
            .iter()
            .map(|range| StatementId::new(&self.content[*range]))
            .collect();
        self.cache.stats(&statements)
    }

    pub fn suppressions(&self) -> &Suppressions {
        &self.suppressions
    }

    /// The ranges of the top-level statements, in file order.
    pub fn statement_ranges(&self) -> &[TextRange] {
        &self.split.ranges
    }

    pub fn get_document_content(&self) -> &str {
        &self.content
    }

    pub fn document_diagnostics(&self) -> &Vec<SDiagnostic> {
        &self.diagnostics
    }

    pub fn find<'a, M>(&'a self, id: StatementId, mapper: M) -> Option<M::Output>
    where
        M: StatementMapper<'a>,
    {
        self.iter_with_filter(mapper, IdFilter::new(id)).next()
    }

    pub fn iter<'a, M>(&'a self, mapper: M) -> ParseIterator<'a, M, NoFilter>
    where
        M: StatementMapper<'a>,
    {
        self.iter_with_filter(mapper, NoFilter)
    }

    pub fn iter_with_filter<'a, M, F>(&'a self, mapper: M, filter: F) -> ParseIterator<'a, M, F>
    where
        M: StatementMapper<'a>,
        F: StatementFilter<'a>,
    {
        ParseIterator::new(self, mapper, filter)
    }

    /// The statements that contain `position`, outermost first: the top-level statement, and the
    /// body of a SQL function if `position` is in it. Unlike filtering all statements with a
    /// [`CursorPositionFilter`], this doesn't look at the other statements of the document.
    pub fn iter_at<'a, M>(
        &'a self,
        mapper: M,
        position: TextSize,
    ) -> ParseIterator<'a, M, CursorPositionFilter>
    where
        M: StatementMapper<'a>,
    {
        // The ranges are ordered and don't overlap, so only the first one that ends after
        // `position` can contain it.
        let first = self
            .split
            .ranges
            .partition_point(|range| range.end() <= position);
        let ranges = self.split.ranges.get(first..=first).unwrap_or_default();
        ParseIterator {
            ranges: ranges.iter(),
            ..ParseIterator::new(self, mapper, CursorPositionFilter::new(position))
        }
    }

    /// The statements from the first top-level statement that starts at or after `offset` on.
    pub fn iter_from<'a, M>(&'a self, mapper: M, offset: TextSize) -> ParseIterator<'a, M, NoFilter>
    where
        M: StatementMapper<'a>,
    {
        let first = self
            .split
            .ranges
            .partition_point(|range| range.start() < offset);
        ParseIterator {
            ranges: self.split.ranges.get(first..).unwrap_or_default().iter(),
            ..ParseIterator::new(self, mapper, NoFilter)
        }
    }

    /// The statements that [`GetCompletionsFilter`] accepts for `position`, without looking at
    /// the statements far from it.
    pub fn iter_for_completions<'a, M>(
        &'a self,
        mapper: M,
        position: TextSize,
    ) -> ParseIterator<'a, M, GetCompletionsFilter>
    where
        M: StatementMapper<'a>,
    {
        // The filter accepts a statement up to two characters after its end. The bodies of SQL
        // functions are inside their top-level statement.
        let ranges = &self.split.ranges;
        let first = ranges.partition_point(|range| range.end() + TextSize::from(2) <= position);
        let near = ranges.get(first..).unwrap_or_default();
        let near = near
            .get(..near.partition_point(|range| range.start() <= position))
            .unwrap_or_default();
        ParseIterator {
            ranges: near.iter(),
            ..ParseIterator::new(
                self,
                mapper,
                GetCompletionsFilter {
                    cursor_position: position,
                },
            )
        }
    }
}

/// A statement of a document: a top-level statement, or a statement embedded in one, like a
/// statement of a SQL function body.
#[derive(Debug, Clone)]
pub struct DocumentStatement {
    pub id: StatementId,
    /// Where the statement is in the document.
    pub range: TextRange,
    /// Maps offsets in the statement text, `id.content()`, to offsets in the document, relative
    /// to the start of `range`. They only differ for embedded statements whose text isn't the
    /// source text, like a function body with escaped quotes.
    pub offsets: OffsetMap,
}

impl DocumentStatement {
    fn top_level(id: StatementId, range: TextRange) -> Self {
        Self {
            id,
            range,
            offsets: OffsetMap::default(),
        }
    }

    /// The range in the document of `range` in the statement text. Like [`Self::statement_offset`],
    /// it can end after the statement text.
    pub fn document_range(&self, range: TextRange) -> TextRange {
        self.offsets.range_to_source(range) + self.range.start()
    }

    /// The offset in the statement text of `offset` in the document, which is in the
    /// statement's range, or up to two bytes after the end of an unterminated statement, where
    /// [`GetCompletionsFilter`] finds the cursor of a completion.
    pub fn statement_offset(&self, offset: TextSize) -> TextSize {
        debug_assert!(self.range.start() <= offset);
        self.offsets
            .to_statement(offset.checked_sub(self.range.start()).unwrap_or_default())
    }
}

pub trait StatementMapper<'a> {
    type Output;

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output;
}

pub trait StatementFilter<'a> {
    fn predicate(&self, stmt: &DocumentStatement) -> bool;
}

pub struct ParseIterator<'a, M, F> {
    parser: &'a Document,
    mapper: M,
    filter: F,
    ranges: std::slice::Iter<'a, TextRange>,
    /// The embedded statements of the last top-level statement, last first.
    pending_embedded: Vec<DocumentStatement>,
}

impl<'a, M, F> ParseIterator<'a, M, F> {
    pub fn new(parser: &'a Document, mapper: M, filter: F) -> Self {
        Self {
            parser,
            mapper,
            filter,
            ranges: parser.split.ranges.iter(),
            pending_embedded: Vec::new(),
        }
    }
}

impl<'a, M, F> Iterator for ParseIterator<'a, M, F>
where
    M: StatementMapper<'a>,
    F: StatementFilter<'a>,
{
    type Item = M::Output;

    fn next(&mut self) -> Option<Self::Item> {
        // A loop rather than recursion: the filter can skip thousands of statements in a row.
        loop {
            if let Some(stmt) = self.pending_embedded.pop() {
                if self.filter.predicate(&stmt) {
                    return Some(self.mapper.map(self.parser, stmt));
                }
                continue;
            }

            let range = *self.ranges.next()?;
            let root_id = StatementId::new(&self.parser.content[range]);

            // The statements of a SQL function body follow the function. They are pushed in
            // reverse, so that they are popped in order.
            let embedded = self.parser.cache.embedded(&root_id);
            self.pending_embedded
                .extend(embedded.iter().rev().map(|embedded| DocumentStatement {
                    id: root_id.create_child(&embedded.sql),
                    range: embedded.range + range.start(),
                    offsets: embedded.offsets.clone(),
                }));

            let stmt = DocumentStatement::top_level(root_id, range);
            if self.filter.predicate(&stmt) {
                return Some(self.mapper.map(self.parser, stmt));
            }
        }
    }
}

pub struct DefaultMapper;
impl<'a> StatementMapper<'a> for DefaultMapper {
    type Output = (StatementId, TextRange, String);

    fn map(&self, _doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let sql = stmt.id.content().to_string();
        (stmt.id, stmt.range, sql)
    }
}

pub struct ExecuteStatementMapper;
impl<'a> StatementMapper<'a> for ExecuteStatementMapper {
    type Output = (
        StatementId,
        TextRange,
        String,
        Option<Arc<pgls_query::NodeEnum>>,
    );

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let ast_result = doc.cache.parse(&stmt.id);
        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        let sql = stmt.id.content().to_string();
        (stmt.id, stmt.range, sql, ast_option)
    }
}

pub struct FormatStatementMapper;
impl<'a> StatementMapper<'a> for FormatStatementMapper {
    type Output = (
        StatementId,
        TextRange,
        String,
        Result<Arc<pgls_query::NodeEnum>, pgls_query_ext::diagnostics::SyntaxDiagnostic>,
    );

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let ast_result = doc.cache.parse(&stmt.id);
        let result = match &*ast_result {
            Ok(node) => Ok(node.clone()),
            Err(diag) => Err(diag.clone().span(stmt.range)),
        };

        let sql = stmt.id.content().to_string();
        (stmt.id, stmt.range, sql, result)
    }
}

pub struct TypecheckDiagnosticsMapper;
impl<'a> StatementMapper<'a> for TypecheckDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        Option<Arc<pgls_query::NodeEnum>>,
        Arc<tree_sitter::Tree>,
    );

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let ast_result = doc.cache.parse(&stmt.id);

        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        let cst_result = doc.cache.tree(&stmt.id);

        (stmt.id, stmt.range, ast_option, cst_result)
    }
}

pub struct AnalyserDiagnosticsMapper;
impl<'a> StatementMapper<'a> for AnalyserDiagnosticsMapper {
    /// The statement if it parses, its syntax error, and the statement again if it is a
    /// top-level statement that doesn't parse.
    type Output = (
        Option<AnalysableStatement>,
        Option<SyntaxDiagnostic>,
        Option<UnparsableStatement>,
    );

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let DocumentStatement { id, range, offsets } = stmt;
        let maybe_node = doc.cache.parse(&id);

        let (ast_option, diagnostics) = match &*maybe_node {
            Ok(node) => {
                let plpgsql_result = doc.cache.plpgsql_parse(&id);
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
        let unparsable =
            (ast_option.is_none() && id.parent().is_none()).then(|| UnparsableStatement {
                range,
                sql: id.content().to_owned(),
            });

        // Statements in the body of a SQL function can reference its parameters.
        let function = id.parent().and_then(|root| {
            let parent = doc.cache.parse(&root);
            let Ok(pgls_query::NodeEnum::CreateFunctionStmt(create_fn)) =
                parent.as_ref().as_deref()
            else {
                return None;
            };
            pgls_catalog::resolve::FunctionContext::from_create_function(create_fn)
        });

        (
            ast_option.map(|root| {
                AnalysableStatement {
                    offsets,
                    ..AnalysableStatement::new(root, range).with_sql(id.content())
                }
                .with_function(function)
                .with_identifier_parameters(
                    pgls_lexer::convert_to_positional_params_with_metadata(id.content())
                        .has_identifier_parameters,
                )
            }),
            diagnostics,
            unparsable,
        )
    }
}
pub struct WithCSTMapper;
impl<'a> StatementMapper<'a> for WithCSTMapper {
    type Output = (DocumentStatement, Arc<tree_sitter::Tree>);

    fn map(&self, doc: &'a Document, stmt: DocumentStatement) -> Self::Output {
        let tree = doc.cache.tree(&stmt.id);
        (stmt, tree)
    }
}

/*
 * We allow an offset of two for the statement:
 *
 * select * from | <-- we want to suggest items for the next token.
 *
 * However, if the current statement is terminated by a semicolon, we don't apply any
 * offset.
 *
 * select * from users; | <-- no autocompletions here.
 */
pub struct GetCompletionsFilter {
    pub cursor_position: TextSize,
}
impl StatementFilter<'_> for GetCompletionsFilter {
    fn predicate(&self, stmt: &DocumentStatement) -> bool {
        let range = stmt.range;
        let is_terminated_by_semi = stmt.id.content().ends_with(';');

        let measuring_range = if is_terminated_by_semi {
            range
        } else {
            range.checked_expand_end(2.into()).unwrap_or(range)
        };
        measuring_range.contains(self.cursor_position)
    }
}

pub struct NoFilter;
impl StatementFilter<'_> for NoFilter {
    fn predicate(&self, _stmt: &DocumentStatement) -> bool {
        true
    }
}

pub struct CursorPositionFilter {
    pos: TextSize,
}

impl CursorPositionFilter {
    pub fn new(pos: TextSize) -> Self {
        Self { pos }
    }
}

impl StatementFilter<'_> for CursorPositionFilter {
    fn predicate(&self, stmt: &DocumentStatement) -> bool {
        stmt.range.contains(self.pos)
    }
}

pub struct IdFilter {
    id: StatementId,
}

impl IdFilter {
    pub fn new(id: StatementId) -> Self {
        Self { id }
    }
}

impl StatementFilter<'_> for IdFilter {
    fn predicate(&self, stmt: &DocumentStatement) -> bool {
        stmt.id == self.id
    }
}

/// The length of the longest common prefix of `a` and `b`, on a character boundary.
fn common_prefix(a: &str, b: &str) -> TextSize {
    /// Compares this many bytes at a time, which is faster than comparing each byte.
    const BLOCK_BYTES: usize = 64;
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let blocks = a
        .as_chunks::<BLOCK_BYTES>()
        .0
        .iter()
        .zip(b.as_chunks::<BLOCK_BYTES>().0)
        .take_while(|(a, b)| a == b)
        .count();
    let mut equal = blocks * BLOCK_BYTES;
    equal += a
        .iter()
        .skip(equal)
        .zip(b.iter().skip(equal))
        .take_while(|(a, b)| a == b)
        .count();
    // A change inside a character starts at its first byte.
    while equal > 0 && a.get(equal).is_some_and(|byte| (byte & 0xC0) == 0x80) {
        equal -= 1;
    }
    TextSize::try_from(equal).unwrap_or_default()
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

    #[test]
    fn common_prefix_stops_at_the_first_difference_on_a_char_boundary() {
        let long = "select 1;\n".repeat(20);
        assert_eq!(common_prefix("", ""), TextSize::from(0));
        assert_eq!(common_prefix(&long, &long), TextSize::of(long.as_str()));
        assert_eq!(common_prefix(&long, "select 2;"), TextSize::from(7));
        assert_eq!(
            common_prefix(&format!("{long}x"), &format!("{long}y")),
            TextSize::of(long.as_str())
        );
        // `ä` and `ö` share their first byte, so the change starts at that byte.
        assert_eq!(
            common_prefix(&format!("{long}ä"), &format!("{long}ö")),
            TextSize::of(long.as_str())
        );
    }

    #[test]
    fn sql_function_body() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let d = Document::new(input.to_string(), 1);

        let stmts = d.iter(DefaultMapper).collect::<Vec<_>>();

        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[1].2, "select $1 + $2;");
    }

    #[test]
    fn sql_function_body_sees_the_parameters() {
        let input =
            "CREATE FUNCTION add(a int, b int) RETURNS int AS 'SELECT a + $2;' LANGUAGE sql;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 2);
        assert!(results[0].0.as_ref().unwrap().function.is_none());
        let body = results[1].0.as_ref().unwrap();
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

        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 3);
        for (statement, syntax_error, _) in &results[1..] {
            assert!(syntax_error.is_none(), "{syntax_error:?}");
            let statement = statement.as_ref().unwrap();
            assert!(statement.function.is_some());
            assert_eq!(
                Some(&input[statement.range]),
                statement.sql.as_deref(),
                "the range is the statement's range in the document"
            );
        }
        assert_eq!(
            results[1].0.as_ref().unwrap().sql.as_deref(),
            Some("insert into t values (a);")
        );
        assert_eq!(
            results[2].0.as_ref().unwrap().sql.as_deref(),
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

            let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

            assert_eq!(results.len(), 2, "{input}");
            for (_, syntax_error, _) in &results {
                assert!(syntax_error.is_none(), "{input}: {syntax_error:?}");
            }
        }
    }

    #[test]
    fn empty_sql_function_body_has_no_syntax_error() {
        let input = "create function f() returns void language sql as $$ $$;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_none());
    }

    #[test]
    fn test_sync_diagnostics_mapper_plpgsql_syntax_error() {
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
        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        let (ast, diagnostic, _) = &results[0];

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
    fn test_sync_diagnostics_mapper_plpgsql_valid() {
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
        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        let (ast, diagnostic, _) = &results[0];

        // Should have parsed the CREATE FUNCTION statement
        assert!(ast.is_some());

        // Should NOT have any PL/pgSQL syntax errors
        assert!(diagnostic.is_none());
    }

    #[test]
    fn test_sync_diagnostics_mapper_plpgsql_caching() {
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

        let results1 = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();
        assert_eq!(results1.len(), 1);
        assert!(results1[0].0.is_some());
        assert!(results1[0].1.is_none());

        let results2 = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();
        assert_eq!(results2.len(), 1);
        assert!(results2[0].0.is_some());
        assert!(results2[0].1.is_none());
    }

    #[test]
    fn test_default_mapper() {
        let input = "SELECT 1; INSERT INTO users VALUES (1);";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(DefaultMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].2, "SELECT 1;");
        assert_eq!(results[1].2, "INSERT INTO users VALUES (1);");

        assert_eq!(results[0].1.start(), 0.into());
        assert_eq!(results[0].1.end(), 9.into());
        assert_eq!(results[1].1.start(), 10.into());
        assert_eq!(results[1].1.end(), 39.into());
    }

    #[test]
    fn hstore_delete_function_has_no_syntax_diagnostic() {
        let input = "SELECT delete(resource_attributes, 'gen_ai.system');";
        let d = Document::new(input.to_string(), 1);

        assert!(d.document_diagnostics().is_empty());

        let results = d.iter(AnalyserDiagnosticsMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert!(results[0].0.is_some());
        assert!(results[0].1.is_none());
    }

    #[test]
    fn test_execute_statement_mapper() {
        let input = "SELECT 1; INVALID SYNTAX HERE;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(ExecuteStatementMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        // First statement should parse successfully
        assert_eq!(results[0].2, "SELECT 1;");
        assert!(results[0].3.is_some());

        // Second statement should fail to parse
        assert_eq!(results[1].2, "INVALID SYNTAX HERE;");
        assert!(results[1].3.is_none());
    }

    #[test]
    fn test_async_diagnostics_mapper() {
        let input = "
CREATE FUNCTION test_fn() RETURNS integer AS $$
BEGIN
    RETURN 42;
END;
$$ LANGUAGE plpgsql;";

        let d = Document::new(input.to_string(), 1);
        let results = d.iter(TypecheckDiagnosticsMapper).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        let (_id, _range, ast, cst) = &results[0];

        // Should have both AST and CST
        assert!(ast.is_some());
        assert_eq!(cst.root_node().kind(), "program");
    }

    #[test]
    fn test_async_diagnostics_mapper_with_sql_function_body() {
        let input =
            "CREATE FUNCTION add(a int, b int) RETURNS int AS 'SELECT $1 + $2;' LANGUAGE sql;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(TypecheckDiagnosticsMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        // Check the function body
        let (id, _range, ast, _cst) = &results[1];
        assert_eq!(id.content(), "SELECT $1 + $2;");
        assert!(ast.is_some());
    }

    #[test]
    fn test_get_completions_mapper() {
        let input = "SELECT * FROM users;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(WithCSTMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let (stmt, tree) = &results[0];
        assert_eq!(stmt.id.content(), "SELECT * FROM users;");
        assert_eq!(tree.root_node().kind(), "program");
    }

    #[test]
    fn test_get_completions_filter() {
        let input = "SELECT * FROM users; INSERT INTO";
        let d = Document::new(input.to_string(), 1);

        // Test cursor at end of first statement (terminated with semicolon)
        let filter1 = GetCompletionsFilter {
            cursor_position: 20.into(),
        };
        let results1 = d
            .iter_with_filter(DefaultMapper, filter1)
            .collect::<Vec<_>>();
        assert_eq!(results1.len(), 0); // No completions after semicolon

        // Test cursor at end of second statement (not terminated)
        let filter2 = GetCompletionsFilter {
            cursor_position: 32.into(),
        };
        let results2 = d
            .iter_with_filter(DefaultMapper, filter2)
            .collect::<Vec<_>>();
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].2, "INSERT INTO");
    }

    #[test]
    fn test_cursor_position_filter() {
        let input = "SELECT 1; INSERT INTO users VALUES (1);";
        let d = Document::new(input.to_string(), 1);

        // Cursor in first statement
        let filter1 = CursorPositionFilter::new(5.into());
        let results1 = d
            .iter_with_filter(DefaultMapper, filter1)
            .collect::<Vec<_>>();
        assert_eq!(results1.len(), 1);
        assert_eq!(results1[0].2, "SELECT 1;");

        // Cursor in second statement
        let filter2 = CursorPositionFilter::new(25.into());
        let results2 = d
            .iter_with_filter(DefaultMapper, filter2)
            .collect::<Vec<_>>();
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].2, "INSERT INTO users VALUES (1);");
    }

    #[test]
    fn iter_at_yields_the_statements_at_the_position() {
        let input =
            "select 1;\ncreate function f() returns int language sql as 'select 2;';\nselect 3;";
        let d = Document::new(input.to_string(), 1);
        let at = |needle: &str| {
            let position = TextSize::try_from(input.find(needle).unwrap()).unwrap();
            d.iter_at(DefaultMapper, position)
                .map(|(_, _, sql)| sql)
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
        assert!(
            d.iter_at(DefaultMapper, TextSize::of(input))
                .next()
                .is_none()
        );
    }

    #[test]
    fn keeps_the_parsed_statements_that_are_still_in_the_document() {
        let mut d = Document::new("select 1;\nselect 2;".to_string(), 1);
        let first = StatementId::new("select 1;");
        let second = StatementId::new("select 2;");
        let first_ast = d.cache.parse(&first);
        let second_ast = d.cache.parse(&second);
        let first_tree = d.cache.tree(&first);

        d.update_content("select 1;\nselect 3;".to_string(), 2)
            .drop_in_background();

        assert!(Arc::ptr_eq(&first_ast, &d.cache.parse(&first)));
        assert!(Arc::ptr_eq(&first_tree, &d.cache.tree(&first)));
        assert!(!Arc::ptr_eq(&second_ast, &d.cache.parse(&second)));
    }

    #[test]
    fn test_id_filter() {
        let input = "SELECT 1; SELECT 2;";
        let d = Document::new(input.to_string(), 1);

        // Get all statements first to get their IDs
        let all_results = d.iter(DefaultMapper).collect::<Vec<_>>();
        assert_eq!(all_results.len(), 2);

        // Filter by first statement ID
        let filter = IdFilter::new(all_results[0].0.clone());
        let results = d
            .iter_with_filter(DefaultMapper, filter)
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "SELECT 1;");
    }

    #[test]
    fn test_no_filter() {
        let input = "SELECT 1; SELECT 2; SELECT 3;";
        let d = Document::new(input.to_string(), 1);

        let results = d
            .iter_with_filter(DefaultMapper, NoFilter)
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_find_method() {
        let input = "SELECT 1; SELECT 2;";
        let d = Document::new(input.to_string(), 1);

        // Get all statements to get their IDs
        let all_results = d.iter(DefaultMapper).collect::<Vec<_>>();

        // Find specific statement
        let result = d.find(all_results[1].0.clone(), DefaultMapper);
        assert!(result.is_some());
        assert_eq!(result.unwrap().2, "SELECT 2;");

        // Try to find non-existent statement
        let fake_id = StatementId::new("SELECT 3;");
        let result = d.find(fake_id, DefaultMapper);
        assert!(result.is_none());
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
