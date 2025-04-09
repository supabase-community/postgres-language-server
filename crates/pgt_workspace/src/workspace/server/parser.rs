use std::sync::Arc;

use pgt_fs::PgTPath;
use pgt_text_size::TextRange;

use crate::workspace::ChangeFileParams;

use super::{
    StatementId,
    change::StatementChange,
    document::{Document, StatementIterator},
    pg_query::PgQueryStore,
    sql_function::SQLFunctionBodyStore,
    tree_sitter::TreeSitterStore,
};

pub struct Parser {
    path: PgTPath,

    doc: Document,
    ast_db: PgQueryStore,
    cst_db: TreeSitterStore,
    sql_fn_db: SQLFunctionBodyStore,
}

impl Parser {
    pub fn new(path: PgTPath, content: String, version: i32) -> Parser {
        let doc = Document::new(content, version);

        let cst_db = TreeSitterStore::new();
        let ast_db = PgQueryStore::new();
        let sql_fn_db = SQLFunctionBodyStore::new();

        doc.iter().for_each(|(stmt, _, content)| {
            cst_db.add_statement(&stmt, content);
        });

        Parser {
            path,
            doc,
            ast_db,
            cst_db,
            sql_fn_db,
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
                }
            }
        }
    }

    pub fn iter<'a, M>(&'a self, mapper: M) -> ParseIterator<'a, M, DefaultFilter>
    where
        M: StatementMapper<'a>,
    {
        self.iter_with_filter(mapper, DefaultFilter)
    }

    pub fn iter_with_filter<'a, M, F>(&'a self, mapper: M, filter: F) -> ParseIterator<'a, M, F>
    where
        M: StatementMapper<'a>,
        F: StatementFilter<'a>,
    {
        ParseIterator::new(self, mapper, filter)
    }
}

pub trait StatementMapper<'a> {
    type Output;

    fn map(
        &self,
        parser: &'a Parser,
        id: StatementId,
        range: &TextRange,
        content: &str,
    ) -> Self::Output;
}

pub trait StatementFilter<'a> {
    fn apply(&self, range: &TextRange) -> bool;
}

pub struct ParseIterator<'a, M, F> {
    parser: &'a Parser,
    statements: StatementIterator<'a>,
    mapper: M,
    filter: F,
    pending_sub_statements: Vec<(StatementId, TextRange, &'a str)>,
}

impl<'a, M, F> ParseIterator<'a, M, F> {
    pub fn new(parser: &'a Parser, mapper: M, filter: F) -> Self {
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
            if self.filter.apply(&range) {
                return Some(self.mapper.map(self.parser, id, &range, &content));
            }
            // If the sub-statement doesn't pass the filter, continue to the next item
            return self.next();
        }

        // Process the next top-level statement
        let next_statement = self.statements.next();

        if let Some((root_id, range, content)) = next_statement {
            // If we should include sub-statements and this statement has an AST
            if let Ok(ast) = *self.parser.ast_db.load_parse(&root_id, &content) {
                // Check if this is a SQL function definition with a body
                if let Some(sub_statement) = self
                    .parser
                    .sql_fn_db
                    .get_function_body(&root_id, &ast, content)
                {
                    // Add sub-statements to our pending queue
                    self.pending_sub_statements.push((
                        root_id.create_child(),
                        // adjust range to document
                        sub_statement.range + range.start(),
                        &sub_statement.body,
                    ));
                }
            }

            // Return the current statement if it passes the filter
            if self.filter.apply(&range) {
                return Some(self.mapper.map(self.parser, root_id, &range, content));
            }

            // If the current statement doesn't pass the filter, try the next one
            return self.next();
        }

        None
    }
}

struct WithAst;
impl<'a> StatementMapper<'a> for WithAst {
    type Output = (StatementId, TextRange, Option<Arc<pgt_query_ext::NodeEnum>>);

    fn map(
        &self,
        parser: &'a Parser,
        id: StatementId,
        range: &TextRange,
        _content: &str,
    ) -> Self::Output {
        let ast = parser.ast_db.get_ast(&id);
        (id, *range, ast)
    }
}

struct DefaultFilter;
impl<'a> StatementFilter<'a> for DefaultFilter {
    fn apply(&self, _range: &TextRange) -> bool {
        true
    }
}
