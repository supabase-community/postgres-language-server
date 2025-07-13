use std::sync::Arc;

use pgt_analyser::AnalysableStatement;
use pgt_diagnostics::{Diagnostic, DiagnosticExt, serde::Diagnostic as SDiagnostic};
use pgt_query_ext::diagnostics::SyntaxDiagnostic;
use pgt_suppressions::Suppressions;
use pgt_text_size::{TextRange, TextSize};

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
    type Output = (
        StatementId,
        TextRange,
        String,
        Option<pgt_query_ext::NodeEnum>,
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

pub struct AsyncDiagnosticsMapper;
impl<'a> StatementMapper<'a> for AsyncDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        Option<pgt_query_ext::NodeEnum>,
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

pub struct LintDiagnosticsMapper;
impl<'a> StatementMapper<'a> for LintDiagnosticsMapper {
    type Output = Result<pgt_analyser::AnalysableStatement, SyntaxDiagnostic>;

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let maybe_node = parser.ast_db.get_or_cache_ast(&id);

        match maybe_node.as_ref() {
            Ok(node) => Ok(AnalysableStatement {
                range,
                root: node.clone(),
            }),
            Err(diag) => Err(SyntaxDiagnostic {
                message: diag.message.clone(),
                span: Some(range),
            }),
        }
    }
}

pub struct GetCompletionsMapper;
impl<'a> StatementMapper<'a> for GetCompletionsMapper {
    type Output = (StatementId, TextRange, String, Arc<tree_sitter::Tree>);

    fn map(&self, parser: &'a Document, id: StatementId, range: TextRange) -> Self::Output {
        let tree = parser.cst_db.get_or_cache_tree(&id);
        (id.clone(), range, id.content().to_string(), tree)
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
    let result = pgt_statement_splitter::split(content);

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
}
