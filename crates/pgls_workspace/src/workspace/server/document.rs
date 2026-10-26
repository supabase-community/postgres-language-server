use std::sync::Arc;

use pgls_analyser::{AnalysableStatement, MigrationSection, UnparsableStatement};
use pgls_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgls_query_ext::{diagnostics::SyntaxDiagnostic, utils::sql_function_body};
use pgls_suppressions::Suppressions;
use pgls_text_size::{TextRange, TextSize};
use rustc_hash::FxHashSet;

use crate::{migrations::MigrationTool, workspace::ProjectKey};

use super::{
    annotation::AnnotationStore,
    pg_query::{PgQueryStore, convert_to_positional_params_with_metadata},
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
    /// The project whose settings apply to the document.
    project_key: ProjectKey,
    /// The migration tool that runs the file, from its settings. See [`Document::open`].
    migration_tool: Option<&'static dyn MigrationTool>,
    /// The sections of the content for `migration_tool`.
    migration_sections: Vec<MigrationSection>,
}

impl Document {
    /// A document without a project, and so without a migration tool.
    #[cfg(test)]
    pub fn new(content: String, version: i32) -> Document {
        Self::open(ProjectKey::default(), None, content, version)
    }

    /// A document of the project `project_key`, whose settings make `migration_tool` run the
    /// file. When the settings change, [`Document::set_migration_tool`] applies them.
    pub(crate) fn open(
        project_key: ProjectKey,
        migration_tool: Option<&'static dyn MigrationTool>,
        content: String,
        version: i32,
    ) -> Document {
        let cst_db = TreeSitterStore::new();
        let ast_db = PgQueryStore::new();
        let annotation_db = AnnotationStore::new();
        let (ranges, diagnostics) = split_with_diagnostics(&content, None);
        let suppressions = Suppressions::from(content.as_str()).with_statements(ranges.clone());
        let migration_sections = sections_for(migration_tool, &content);

        Document {
            ranges,
            diagnostics,
            content,
            version,
            ast_db,
            cst_db,
            annotation_db,
            suppressions,
            project_key,
            migration_tool,
            migration_sections,
        }
    }

    pub fn update_content(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
        self.migration_sections = sections_for(self.migration_tool, &self.content);

        let (ranges, diagnostics) = split_with_diagnostics(&self.content, None);

        self.suppressions =
            Suppressions::from(self.content.as_str()).with_statements(ranges.clone());
        self.ranges = ranges;
        self.diagnostics = diagnostics;
        // Keep what is cached for the statements that are still in the document. The bodies
        // of SQL functions are kept with their function.
        let statements: FxHashSet<&str> = self
            .ranges
            .iter()
            .map(|range| &self.content[*range])
            .collect();
        let is_present =
            |id: &StatementId| statements.contains(id.parent_content().unwrap_or(id.content()));
        self.ast_db.retain(is_present);
        self.cst_db.retain(is_present);
        self.annotation_db.retain(is_present);
    }

    /// Applies the migration tool of changed settings.
    pub(crate) fn set_migration_tool(
        &mut self,
        migration_tool: Option<&'static dyn MigrationTool>,
    ) {
        self.migration_tool = migration_tool;
        self.migration_sections = sections_for(migration_tool, &self.content);
    }

    pub(crate) fn project_key(&self) -> ProjectKey {
        self.project_key
    }

    /// The migration tool that runs the file, if its settings name one.
    pub(crate) fn migration_tool(&self) -> Option<&'static dyn MigrationTool> {
        self.migration_tool
    }

    /// The migration sections of the document, like the up and down migrations of dbmate and
    /// goose. Without a migration tool, there are none.
    pub fn migration_sections(&self) -> &[MigrationSection] {
        &self.migration_sections
    }

    pub fn suppressions(&self) -> &Suppressions {
        &self.suppressions
    }

    /// The ranges of the top-level statements, in file order.
    pub fn statement_ranges(&self) -> &[TextRange] {
        &self.ranges
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
        let first = self.ranges.partition_point(|range| range.end() <= position);
        let ranges = &self.ranges[first..self.ranges.len().min(first + 1)];
        ParseIterator {
            ranges: ranges.iter(),
            ..ParseIterator::new(self, mapper, CursorPositionFilter::new(position))
        }
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
                // The statements of a SQL function body follow the function. They are pushed in
                // reverse, so that they are popped in order.
                if let Some(body) = sql_function_body(ast, content) {
                    let body_start = range.start() + body.range.start();
                    for statement in body.statements().into_iter().rev() {
                        let text = &body.body[statement];
                        self.pending_sub_statements.push((
                            root_id.create_child(text),
                            statement + body_start,
                            text.to_owned(),
                        ));
                    }
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

pub struct FormatStatementMapper;
impl<'a> StatementMapper<'a> for FormatStatementMapper {
    type Output = (
        StatementId,
        TextRange,
        String,
        Result<pgls_query::NodeEnum, pgls_query_ext::diagnostics::SyntaxDiagnostic>,
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
        Option<pgls_query::NodeEnum>,
        Arc<tree_sitter::Tree>,
    );

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id);

        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        let cst_result = parser.cst_db.get_or_cache_tree(&id);

        (id.clone(), range, ast_option, cst_result)
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
            let Ok(pgls_query::NodeEnum::CreateFunctionStmt(create_fn)) = parent.as_ref() else {
                return None;
            };
            pgls_catalog::resolve::FunctionContext::from_create_function(create_fn)
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

/// The migration sections of `content`, see [`MigrationTool::sections`].
fn sections_for(
    migration_tool: Option<&'static dyn MigrationTool>,
    content: &str,
) -> Vec<MigrationSection> {
    migration_tool.map_or_else(Vec::new, |tool| tool.sections(content))
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
