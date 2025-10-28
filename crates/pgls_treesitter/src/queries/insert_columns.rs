use std::sync::LazyLock;
use tree_sitter::StreamingIterator;

use crate::queries::{Query, QueryResult};

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (insert_columns
        (column_identifier) @column
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

#[derive(Debug)]
pub struct InsertColumnMatch<'a> {
    pub(crate) column: tree_sitter::Node<'a>,
}

impl InsertColumnMatch<'_> {
    pub fn get_column(&self, sql: &str) -> String {
        self.column
            .utf8_text(sql.as_bytes())
            .expect("Failed to get column from ColumnMatch")
            .to_string()
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a InsertColumnMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::InsertClauseColumns(c) => Ok(c),

            #[allow(unreachable_patterns)]
            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for InsertColumnMatch<'a> {
    type Ref = &'a InsertColumnMatch<'a>;
}

impl<'a> Query<'a> for InsertColumnMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            if m.captures.len() == 1 {
                let capture = m.captures[0].node;
                to_return.push(QueryResult::InsertClauseColumns(InsertColumnMatch {
                    column: capture,
                }));
            }
        });
        {}

        to_return
    }
}
#[cfg(test)]
mod tests {
    use super::InsertColumnMatch;
    use crate::queries::TreeSitterQueriesExecutor;

    #[test]
    fn finds_all_insert_columns() {
        let sql = r#"insert into users (id, email, name) values (1, 'a@b.com', 'Alice');"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<InsertColumnMatch>();

        let results: Vec<&InsertColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        let columns: Vec<String> = results.iter().map(|c| c.get_column(sql)).collect();

        assert_eq!(columns, vec!["id", "email", "name"]);
    }

    #[test]
    fn finds_insert_columns_with_whitespace_and_commas() {
        let sql = r#"
            insert into users (
                id,
                email,
                name
            ) values (1, 'a@b.com', 'Alice');
        "#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<InsertColumnMatch>();

        let results: Vec<&InsertColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        let columns: Vec<String> = results.iter().map(|c| c.get_column(sql)).collect();

        assert_eq!(columns, vec!["id", "email", "name"]);
    }

    #[test]
    fn returns_empty_for_insert_without_columns() {
        let sql = r#"insert into users values (1, 'a@b.com', 'Alice');"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<InsertColumnMatch>();

        let results: Vec<&InsertColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert!(results.is_empty());
    }
}
