use std::sync::Arc;

use pgls_analyser::AnalysableStatement;
use pgls_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_suppressions::Suppressions;
use pgls_text_size::{TextRange, TextSize};

use super::{
    annotation::AnnotationStore,
    pg_query::PgQueryStore,
    sql_function::{SQLFunctionSignature, get_sql_fn_body, get_sql_fn_signature},
    statement_identifier::StatementId,
    tree_sitter::TreeSitterStore,
};

pub struct Document {
    content: String,
    version: i32,
    ranges: Vec<TextRange>,
    diagnostics: Vec<SDiagnostic>,
    ast_db: PgQueryStore,
    cst_db: TreeSitterStore,
    #[allow(dead_code)]
    annotation_db: AnnotationStore,
    suppressions: Suppressions,
}

impl Document {
    pub fn new(content: String, version: i32) -> Document {
        let cst_db = TreeSitterStore::new();
        let ast_db = PgQueryStore::new();
        let annotation_db = AnnotationStore::new();
        let suppressions = Suppressions::from(content.as_str());

        let (ranges, diagnostics) = split_with_diagnostics(&content, None);

        Document {
            ranges,
            diagnostics,
            content,
            version,
            ast_db,
            cst_db,
            annotation_db,
            suppressions,
        }
    }

    pub fn update_content(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;

        let (ranges, diagnostics) = split_with_diagnostics(&self.content, None);

        self.ranges = ranges;
        self.diagnostics = diagnostics;
        self.suppressions = Suppressions::from(self.content.as_str());
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

    pub fn iter_with_filter<'a, M, F>(&'a self, mapper: M, filter: F) -> ParseIterator<'a, M, F>
    where
        M: StatementMapper<'a>,
        F: StatementFilter<'a>,
    {
        ParseIterator::new(self, mapper, filter)
    }

    #[allow(dead_code)]
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
            ranges: parser.ranges.iter(),
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
        // First check if we have any pending sub-statements to process
        if let Some((id, range, content)) = self.pending_sub_statements.pop() {
            if self.filter.predicate(&id, &range, content.as_str()) {
                return Some(self.mapper.map(self.parser, id, range));
            }
            // If the sub-statement doesn't pass the filter, continue to the next item
            return self.next();
        }

        // Process the next top-level statement
        let next_range = self.ranges.next();

        if let Some(range) = next_range {
            // If we should include sub-statements and this statement has an AST

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

            // If the current statement doesn't pass the filter, try the next one
            return self.next();
        }

        None
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
    type Output = (StatementId, TextRange, String, Option<pgls_query::NodeEnum>);

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id);
        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        (id.clone(), range, id.content().to_string(), ast_option)
    }
}

pub struct TypecheckDiagnosticsMapper;
impl<'a> StatementMapper<'a> for TypecheckDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        Option<pgls_query::NodeEnum>,
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
    type Output = (Option<AnalysableStatement>, Option<SyntaxDiagnostic>);

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

        (
            ast_option.map(|root| AnalysableStatement { range, root }),
            diagnostics,
        )
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

pub struct WithCSTandASTMapper;
impl<'a> StatementMapper<'a> for WithCSTandASTMapper {
    type Output = (
        StatementId,
        TextRange,
        Arc<tree_sitter::Tree>,
        Option<pgls_query::NodeEnum>,
    );

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let tree = parser.cst_db.get_or_cache_tree(&id);
        let ast_result = parser.ast_db.get_or_cache_ast(&id);

        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        (id.clone(), range, tree, ast_option)
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

/// Helper function that wraps the statement splitter and returns the ranges with unified
/// diagnostics
pub(crate) fn split_with_diagnostics(
    content: &str,
    offset: Option<TextSize>,
) -> (Vec<TextRange>, Vec<SDiagnostic>) {
    let o = offset.unwrap_or_else(|| 0.into());
    let result = pgls_statement_splitter::split(content);

    (
        result.ranges,
        result
            .errors
            .into_iter()
            .map(|err| {
                SDiagnostic::new(
                    err.clone()
                        .with_file_span(err.location().span.map(|r| r + o)),
                )
            })
            .collect(),
    )
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
        let (ast, diagnostic) = &results[0];

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
        let (ast, diagnostic) = &results[0];

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
