use std::sync::{Arc, Mutex};

use pgls_analyser::{AnalysableStatement, UnparsableStatement};
use pgls_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_statement_splitter::SplitResult;
use pgls_suppressions::Suppressions;
use pgls_text_size::{TextRange, TextSize};
use rustc_hash::FxHashSet;

use super::{
    pg_query::{PgQueryStore, convert_to_positional_params_with_metadata},
    sql_function::{SQLFunctionSignature, get_sql_fn_body, get_sql_fn_signature},
    statement_identifier::StatementId,
    tree_sitter::TreeSitterStore,
};

pub struct Document {
    content: String,
    version: i32,
    /// The statements of `content`, kept to split the next version of it incrementally.
    split: SplitResult,
    diagnostics: Vec<SDiagnostic>,
    ast_db: PgQueryStore,
    cst_db: TreeSitterStore,
    suppressions: Suppressions,
    /// Cleared when the content changes.
    analysis: Mutex<Option<super::CachedAnalysis>>,
    /// Least recently used first. Kept when the content changes after them.
    walk_checkpoints: Mutex<Vec<super::WalkCheckpoint>>,
    /// Where the last change of the content starts.
    edited_at: Option<TextSize>,
}

impl Document {
    pub fn new(content: String, version: i32) -> Document {
        let cst_db = TreeSitterStore::new();
        let ast_db = PgQueryStore::new();
        let suppressions = Suppressions::from(content.as_str());

        let split = pgls_statement_splitter::split(&content);
        let diagnostics = split_diagnostics(&split);

        Document {
            split,
            diagnostics,
            content,
            version,
            ast_db,
            cst_db,
            suppressions,
            analysis: Mutex::default(),
            walk_checkpoints: Mutex::default(),
            edited_at: None,
        }
    }

    pub fn update_content(&mut self, content: String, version: i32) {
        // Only the statements around the change are split again.
        let split = pgls_statement_splitter::split_edited(&self.content, &self.split, &content);

        // A walk checkpoint stays valid if the text before it and the statements that start
        // before it are the same.
        let unchanged =
            pgls_statement_splitter::common_prefix(self.content.as_bytes(), content.as_bytes());
        let (old_ranges, new_ranges) = (&self.split.ranges, &split.ranges);
        self.walk_checkpoints
            .get_mut()
            .unwrap()
            .retain(|checkpoint| {
                let at = checkpoint.at();
                let before =
                    |ranges: &[TextRange]| ranges.partition_point(|range| range.start() < at);
                usize::from(at) <= unchanged
                    && old_ranges[..before(old_ranges)] == new_ranges[..before(new_ranges)]
            });

        self.split = split;
        self.edited_at = TextSize::try_from(unchanged).ok();
        self.content = content;
        self.version = version;

        self.diagnostics = split_diagnostics(&self.split);
        self.suppressions = Suppressions::from(self.content.as_str());

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
        self.ast_db.retain(is_present);
        self.cst_db.retain(is_present);
        *self.analysis.get_mut().unwrap() = None;
    }

    /// The number of cached parse results and trees of statements, which is what most of the
    /// memory of a document's caches is for.
    pub fn cache_entries(&self) -> usize {
        self.ast_db.len() + self.cst_db.len()
    }

    /// What the caches of the document hold.
    pub fn cache_stats(&self) -> super::DocumentCacheStats {
        let statements: FxHashSet<StatementId> =
            self.iter(DefaultMapper).map(|(id, _, _)| id).collect();
        super::DocumentCacheStats {
            statements: statements.len(),
            parse_results: self.ast_db.len(),
            trees: self.cst_db.len(),
            analysis: self.analysis.lock().unwrap().is_some(),
            walk_checkpoints: self.walk_checkpoints.lock().unwrap().len(),
        }
    }

    /// Drops everything that is cached for the document. It is recomputed when it is needed
    /// again.
    pub fn clear_caches(&self) {
        self.ast_db.clear();
        self.cst_db.clear();
        *self.analysis.lock().unwrap() = None;
        self.walk_checkpoints.lock().unwrap().clear();
    }

    /// Where walks over the statements stopped, see `WorkspaceServer::walk_statements`.
    pub(super) fn walk_checkpoints(&self) -> &Mutex<Vec<super::WalkCheckpoint>> {
        &self.walk_checkpoints
    }

    /// Where the last change of the content starts.
    pub fn edited_at(&self) -> Option<TextSize> {
        self.edited_at
    }

    /// The ranges of the top-level statements.
    pub fn top_level_ranges(&self) -> &[TextRange] {
        &self.split.ranges
    }

    /// The analysis of the document, see `WorkspaceServer::analyse_document_cached`.
    pub(super) fn analysis(&self) -> &Mutex<Option<super::CachedAnalysis>> {
        &self.analysis
    }

    pub fn suppressions(&self) -> &Suppressions {
        &self.suppressions
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

    /// The statements from the first top-level statement that starts at or after `offset` on.
    pub fn iter_from<'a, M>(&'a self, mapper: M, offset: TextSize) -> ParseIterator<'a, M, NoFilter>
    where
        M: StatementMapper<'a>,
    {
        let ranges = &self.split.ranges;
        let first = ranges.partition_point(|range| range.start() < offset);
        ParseIterator {
            ranges: ranges[first..].iter(),
            ..ParseIterator::new(self, mapper, NoFilter)
        }
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
        let ranges = &self.split.ranges;
        let first = ranges.partition_point(|range| range.end() <= position);
        let ranges = &ranges[first..ranges.len().min(first + 1)];
        ParseIterator {
            ranges: ranges.iter(),
            ..ParseIterator::new(self, mapper, CursorPositionFilter::new(position))
        }
    }

    pub fn count(&self) -> usize {
        self.iter(DefaultMapper).count()
    }
}

pub trait StatementMapper<'a> {
    type Output;

    fn map(&self, parsed: &'a Document, id: StatementId, range: TextRange) -> Self::Output;
}

pub trait StatementFilter<'a> {
    fn predicate(&self, id: &StatementId, range: &TextRange, content: &str) -> bool;
}

pub struct ParseIterator<'a, M, F> {
    parser: &'a Document,
    mapper: M,
    filter: F,
    ranges: std::slice::Iter<'a, TextRange>,
    pending_sub_statements: Vec<(StatementId, TextRange, String)>,
}

impl<'a, M, F> ParseIterator<'a, M, F> {
    pub fn new(parser: &'a Document, mapper: M, filter: F) -> Self {
        Self {
            parser,
            mapper,
            filter,
            ranges: parser.split.ranges.iter(),
            pending_sub_statements: Vec::new(),
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
            // First check if we have any pending sub-statements to process
            if let Some((id, range, content)) = self.pending_sub_statements.pop() {
                if self.filter.predicate(&id, &range, content.as_str()) {
                    return Some(self.mapper.map(self.parser, id, range));
                }
                continue;
            }

            // Process the next top-level statement
            let range = self.ranges.next()?;
            let content = &self.parser.content[*range];
            let root_id = StatementId::new(content);

            if let Ok(ast) = self.parser.ast_db.get_or_cache_ast(&root_id).as_ref() {
                // Check if this is a SQL function definition with a body
                if let Some(sub_statement) = get_sql_fn_body(ast, content) {
                    // Add sub-statements to our pending queue
                    self.pending_sub_statements.push((
                        root_id.create_child(&sub_statement.body),
                        // adjust range to document
                        sub_statement.range + range.start(),
                        sub_statement.body.clone(),
                    ));
                }
            }

            // Return the current statement if it passes the filter
            if self.filter.predicate(&root_id, range, content) {
                return Some(self.mapper.map(self.parser, root_id, *range));
            }
        }
    }
}

pub struct DefaultMapper;
impl<'a> StatementMapper<'a> for DefaultMapper {
    type Output = (StatementId, TextRange, String);

    fn map(&self, _parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        (id.clone(), range, id.content().to_string())
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

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id);
        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        (id.clone(), range, id.content().to_string(), ast_option)
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

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id);
        let result = match &*ast_result {
            Ok(node) => Ok(node.clone()),
            Err(diag) => Err(diag.clone().span(range)),
        };

        (id.clone(), range, id.content().to_string(), result)
    }
}

pub struct TypecheckDiagnosticsMapper;
impl<'a> StatementMapper<'a> for TypecheckDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        Option<Arc<pgls_query::NodeEnum>>,
        Arc<tree_sitter::Tree>,
        Option<SQLFunctionSignature>,
    );

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id);

        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        let cst_result = parser.cst_db.get_or_cache_tree(&id);

        let sql_fn_sig = id.parent().and_then(|root| {
            let ast_option = parser.ast_db.get_or_cache_ast(&root).as_ref().clone().ok();

            let ast_option = ast_option.as_ref()?;

            get_sql_fn_signature(ast_option)
        });

        (id.clone(), range, ast_option, cst_result, sql_fn_sig)
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

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let maybe_node = parser.ast_db.get_or_cache_ast(&id);

        let (ast_option, diagnostics) = match &*maybe_node {
            Ok(node) => {
                let plpgsql_result = parser.ast_db.get_or_cache_plpgsql_parse(&id);
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
            let parent = parser.ast_db.get_or_cache_ast(&root);
            let signature = get_sql_fn_signature(parent.as_ref().as_ref().ok()?)?;
            Some(function_context(signature))
        });

        (
            ast_option.map(|root| {
                AnalysableStatement::new(root, range)
                    .with_sql(id.content())
                    .with_function(function)
                    .with_identifier_parameters(
                        convert_to_positional_params_with_metadata(id.content())
                            .has_identifier_parameters,
                    )
            }),
            diagnostics,
            unparsable,
        )
    }
}
fn function_context(signature: SQLFunctionSignature) -> pgls_catalog::resolve::FunctionContext {
    pgls_catalog::resolve::FunctionContext {
        function_name: signature.name,
        params: signature
            .args
            .into_iter()
            .map(|arg| pgls_catalog::resolve::FunctionParam {
                name: arg.name,
                type_schema: arg.type_.schema,
                type_name: arg.type_.name,
                is_array: arg.type_.is_array,
            })
            .collect(),
    }
}

pub struct WithCSTMapper;
impl<'a> StatementMapper<'a> for WithCSTMapper {
    type Output = (StatementId, TextRange, Arc<tree_sitter::Tree>);

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let tree = parser.cst_db.get_or_cache_tree(&id);
        (id.clone(), range, tree)
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
    fn predicate(&self, _id: &StatementId, range: &TextRange, content: &str) -> bool {
        let is_terminated_by_semi = content.chars().last().is_some_and(|c| c == ';');

        let measuring_range = if is_terminated_by_semi {
            *range
        } else {
            range.checked_expand_end(2.into()).unwrap_or(*range)
        };
        measuring_range.contains(self.cursor_position)
    }
}

pub struct NoFilter;
impl StatementFilter<'_> for NoFilter {
    fn predicate(&self, _id: &StatementId, _range: &TextRange, _content: &str) -> bool {
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
    fn predicate(&self, _id: &StatementId, range: &TextRange, _content: &str) -> bool {
        range.contains(self.pos)
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
    fn predicate(&self, id: &StatementId, _range: &TextRange, _content: &str) -> bool {
        *id == self.id
    }
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
        let (_id, _range, ast, cst, sql_fn_sig) = &results[0];

        // Should have both AST and CST
        assert!(ast.is_some());
        assert_eq!(cst.root_node().kind(), "program");

        // Should not have SQL function signature for top-level statement
        assert!(sql_fn_sig.is_none());
    }

    #[test]
    fn test_async_diagnostics_mapper_with_sql_function_body() {
        let input =
            "CREATE FUNCTION add(a int, b int) RETURNS int AS 'SELECT $1 + $2;' LANGUAGE sql;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(TypecheckDiagnosticsMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        // Check the function body
        let (_id, _range, ast, _cst, sql_fn_sig) = &results[1];
        assert_eq!(_id.content(), "SELECT $1 + $2;");
        assert!(ast.is_some());
        assert!(sql_fn_sig.is_some());

        let sig = sql_fn_sig.as_ref().unwrap();
        assert_eq!(sig.name, "add");
        assert_eq!(sig.args.len(), 2);
        assert_eq!(sig.args[0].name, Some("a".to_string()));
        assert_eq!(sig.args[1].name, Some("b".to_string()));
    }

    #[test]
    fn test_get_completions_mapper() {
        let input = "SELECT * FROM users;";
        let d = Document::new(input.to_string(), 1);

        let results = d.iter(WithCSTMapper).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let (id, _, tree) = &results[0];
        assert_eq!(id.content(), "SELECT * FROM users;");
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
        let first_ast = d.ast_db.get_or_cache_ast(&first);
        let second_ast = d.ast_db.get_or_cache_ast(&second);
        let first_tree = d.cst_db.get_or_cache_tree(&first);

        d.update_content("select 1;\nselect 3;".to_string(), 2);

        assert!(Arc::ptr_eq(&first_ast, &d.ast_db.get_or_cache_ast(&first)));
        assert!(Arc::ptr_eq(
            &first_tree,
            &d.cst_db.get_or_cache_tree(&first)
        ));
        assert!(!Arc::ptr_eq(
            &second_ast,
            &d.ast_db.get_or_cache_ast(&second)
        ));
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
            d.update_content(content.clone(), version);

            let whole = pgls_statement_splitter::split(&content);
            assert_eq!(d.split, whole, "{content}");
            assert_eq!(
                format!("{:?}", d.diagnostics),
                format!("{:?}", split_diagnostics(&whole))
            );
        }
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
}
