use std::sync::Arc;

use pgt_diagnostics::serde::Diagnostic as SDiagnostic;
use pgt_fs::PgTPath;
use pgt_query_ext::diagnostics::SyntaxDiagnostic;
use pgt_text_size::{TextRange, TextSize};

use crate::workspace::ChangeFileParams;

use super::{
    annotation::AnnotationStore,
    change::StatementChange,
    document::{Document, StatementIterator},
    pg_query::PgQueryStore,
    sql_function::{SQLFunctionBodyStore, SQLFunctionSignature},
    statement_identifier::StatementId,
    tree_sitter::TreeSitterStore,
};

pub struct ParsedDocument {
    #[allow(dead_code)]
    path: PgTPath,

    doc: Document,
    ast_db: PgQueryStore,
    cst_db: TreeSitterStore,
    sql_fn_db: SQLFunctionBodyStore,
    annotation_db: AnnotationStore,
}

impl ParsedDocument {
    pub fn new(path: PgTPath, content: String, version: i32) -> ParsedDocument {
        let doc = Document::new(content, version);

        let cst_db = TreeSitterStore::new();
        let ast_db = PgQueryStore::new();
        let sql_fn_db = SQLFunctionBodyStore::new();
        let annotation_db = AnnotationStore::new();

        doc.iter().for_each(|(stmt, _, content)| {
            cst_db.add_statement(&stmt, content);
        });

        ParsedDocument {
            path,
            doc,
            ast_db,
            cst_db,
            sql_fn_db,
            annotation_db,
        }
    }

    /// Applies a change to the document and updates the CST and AST databases accordingly.
    ///
    /// Note that only tree-sitter cares about statement modifications vs remove + add.
    /// Hence, we just clear the AST for the old statements and lazily load them when requested.
    ///
    /// * `params`: ChangeFileParams - The parameters for the change to be applied.
    pub fn apply_change(&mut self, params: ChangeFileParams) {
        for c in &self.doc.apply_file_change(&params) {
            match c {
                StatementChange::Added(added) => {
                    tracing::debug!(
                        "Adding statement: id:{:?}, text:{:?}",
                        added.stmt,
                        added.text
                    );
                    self.cst_db.add_statement(&added.stmt, &added.text);
                }
                StatementChange::Deleted(s) => {
                    tracing::debug!("Deleting statement: id {:?}", s,);
                    self.cst_db.remove_statement(s);
                    self.ast_db.clear_statement(s);
                    self.sql_fn_db.clear_statement(s);
                    self.annotation_db.clear_statement(s);
                }
                StatementChange::Modified(s) => {
                    tracing::debug!(
                        "Modifying statement with id {:?} (new id {:?}). Range {:?}, Changed from '{:?}' to '{:?}', changed text: {:?}",
                        s.old_stmt,
                        s.new_stmt,
                        s.change_range,
                        s.old_stmt_text,
                        s.new_stmt_text,
                        s.change_text
                    );

                    self.cst_db.modify_statement(s);
                    self.ast_db.clear_statement(&s.old_stmt);
                    self.sql_fn_db.clear_statement(&s.old_stmt);
                    self.annotation_db.clear_statement(&s.old_stmt);
                }
            }
        }
    }

    pub fn get_document_content(&self) -> &str {
        &self.doc.content
    }

    pub fn document_diagnostics(&self) -> &Vec<SDiagnostic> {
        &self.doc.diagnostics
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

    fn map(
        &self,
        parsed: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output;
}

pub trait StatementFilter<'a> {
    fn predicate(&self, id: &StatementId, range: &TextRange, content: &str) -> bool;
}

pub struct ParseIterator<'a, M, F> {
    parser: &'a ParsedDocument,
    statements: StatementIterator<'a>,
    mapper: M,
    filter: F,
    pending_sub_statements: Vec<(StatementId, TextRange, String)>,
}

impl<'a, M, F> ParseIterator<'a, M, F> {
    pub fn new(parser: &'a ParsedDocument, mapper: M, filter: F) -> Self {
        Self {
            parser,
            statements: parser.doc.iter(),
            mapper,
            filter,
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
                return Some(self.mapper.map(self.parser, id, range, &content));
            }
            // If the sub-statement doesn't pass the filter, continue to the next item
            return self.next();
        }

        // Process the next top-level statement
        let next_statement = self.statements.next();

        if let Some((root_id, range, content)) = next_statement {
            // If we should include sub-statements and this statement has an AST
            let content_owned = content.to_string();
            if let Ok(ast) = self
                .parser
                .ast_db
                .get_or_cache_ast(&root_id, &content_owned)
                .as_ref()
            {
                // Check if this is a SQL function definition with a body
                if let Some(sub_statement) =
                    self.parser
                        .sql_fn_db
                        .get_function_body(&root_id, ast, &content_owned)
                {
                    // Add sub-statements to our pending queue
                    self.pending_sub_statements.push((
                        root_id.create_child(),
                        // adjust range to document
                        sub_statement.range + range.start(),
                        sub_statement.body.clone(),
                    ));
                }
            }

            // Return the current statement if it passes the filter
            if self.filter.predicate(&root_id, &range, content) {
                return Some(self.mapper.map(self.parser, root_id, range, content));
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

    fn map(
        &self,
        _parser: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output {
        (id, range, content.to_string())
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

    fn map(
        &self,
        parser: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id, content);
        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        (id, range, content.to_string(), ast_option)
    }
}

pub struct AsyncDiagnosticsMapper;
impl<'a> StatementMapper<'a> for AsyncDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        String,
        Option<pgt_query_ext::NodeEnum>,
        Arc<tree_sitter::Tree>,
        Option<Arc<SQLFunctionSignature>>,
    );

    fn map(
        &self,
        parser: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output {
        let content_owned = content.to_string();
        let ast_result = parser.ast_db.get_or_cache_ast(&id, &content_owned);

        let ast_option = match &*ast_result {
            Ok(node) => Some(node.clone()),
            Err(_) => None,
        };

        let cst_result = parser.cst_db.get_or_cache_tree(&id, &content_owned);

        let sql_fn_sig = id
            .parent()
            .and_then(|root| {
                let c = parser.doc.statement_content(&root)?;
                Some((root, c))
            })
            .and_then(|(root, c)| {
                let ast_option = parser
                    .ast_db
                    .get_or_cache_ast(&root, c)
                    .as_ref()
                    .clone()
                    .ok();

                let ast_option = ast_option.as_ref()?;

                parser.sql_fn_db.get_function_signature(&root, ast_option)
            });

        (id, range, content_owned, ast_option, cst_result, sql_fn_sig)
    }
}

pub struct SyncDiagnosticsMapper;
impl<'a> StatementMapper<'a> for SyncDiagnosticsMapper {
    type Output = (
        StatementId,
        TextRange,
        Option<pgt_query_ext::NodeEnum>,
        Option<SyntaxDiagnostic>,
    );

    fn map(
        &self,
        parser: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output {
        let ast_result = parser.ast_db.get_or_cache_ast(&id, content);

        let (ast_option, diagnostics) = match &*ast_result {
            Ok(node) => (Some(node.clone()), None),
            Err(diag) => (None, Some(diag.clone())),
        };

        (id, range, ast_option, diagnostics)
    }
}

pub struct GetCompletionsMapper;
impl<'a> StatementMapper<'a> for GetCompletionsMapper {
    type Output = (StatementId, TextRange, String, Arc<tree_sitter::Tree>);

    fn map(
        &self,
        parser: &'a ParsedDocument,
        id: StatementId,
        range: TextRange,
        content: &str,
    ) -> Self::Output {
        let tree = parser.cst_db.get_or_cache_tree(&id, content);
        (id, range, content.into(), tree)
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
impl<'a> StatementFilter<'a> for GetCompletionsFilter {
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
impl<'a> StatementFilter<'a> for NoFilter {
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

impl<'a> StatementFilter<'a> for CursorPositionFilter {
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

impl<'a> StatementFilter<'a> for IdFilter {
    fn predicate(&self, id: &StatementId, _range: &TextRange, _content: &str) -> bool {
        *id == self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pgt_fs::PgTPath;

    #[test]
    fn sql_function_body() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let path = PgTPath::new("test.sql");

        let d = ParsedDocument::new(path, input.to_string(), 0);

        let stmts = d.iter(DefaultMapper).collect::<Vec<_>>();

        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[1].2, "select $1 + $2;");
    }
}
