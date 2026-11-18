use std::sync::LazyLock;

use crate::queries::{Query, QueryResult, object_references::parts_of_reference_query};

use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static WHERE_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (where) @where
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

/**
 * The binary expressions can be nested inside a @where clause, e.g. (where user_id = 1 or (email = 2 and user_id = 3));
 * We'll need a separate query to find all nested binary expressions.
 */
static BINARY_EXPR_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (binary_expression 
        binary_expr_left: (object_reference) @ref
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

#[derive(Debug)]
pub struct WhereColumnMatch<'a> {
    pub(crate) alias: Option<tree_sitter::Node<'a>>,
    pub(crate) column: tree_sitter::Node<'a>,
}

impl WhereColumnMatch<'_> {
    pub fn get_alias(&self, sql: &str) -> Option<String> {
        Some(
            self.alias
                .as_ref()?
                .utf8_text(sql.as_bytes())
                .ok()?
                .to_string(),
        )
    }

    pub fn get_column(&self, sql: &str) -> String {
        self.column
            .utf8_text(sql.as_bytes())
            // A node of `sql` is UTF-8, unless the tree is stale.
            .unwrap_or_default()
            .to_string()
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a WhereColumnMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::WhereClauseColumns(c) => Ok(c),

            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for WhereColumnMatch<'a> {
    type Ref = &'a WhereColumnMatch<'a>;
}

impl<'a> Query<'a> for WhereColumnMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let (Some(where_query), Some(binary_expr_query)) =
            (WHERE_QUERY.as_ref(), BINARY_EXPR_QUERY.as_ref())
        else {
            return vec![];
        };
        let where_matches = cursor.matches(where_query, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        where_matches.for_each(|where_match| {
            let Some(where_clause) = where_match.captures.first() else {
                return;
            };
            let mut binary_cursor = tree_sitter::QueryCursor::new();

            let binary_expr_matches =
                binary_cursor.matches(binary_expr_query, where_clause.node, stmt.as_bytes());

            binary_expr_matches.for_each(|m| {
                if let [capture] = m.captures {
                    let capture = capture.node;

                    if let Some((_, alias, column)) = parts_of_reference_query(capture, stmt) {
                        to_return.push(QueryResult::WhereClauseColumns(WhereColumnMatch {
                            alias,
                            column,
                        }));
                    }
                }
            })
        });

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::WhereColumnMatch;

    #[test]
    fn finds_column_without_alias() {
        let sql = r#"select * from users where id = 1;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<WhereColumnMatch>();

        let results: Vec<&WhereColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_column(sql), "id");
        assert_eq!(results[0].get_alias(sql), None);
    }

    #[test]
    fn finds_column_with_table_alias() {
        let sql = r#"select * from users u where u.id = 1;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<WhereColumnMatch>();

        let results: Vec<&WhereColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_column(sql), "id");
        assert_eq!(results[0].get_alias(sql), Some("u".into()));
    }

    #[test]
    fn finds_multiple_columns_in_where_clause() {
        let sql = r#"
select * from users u
join posts p on u.id = p.user_id
where u.email = 'test@example.com' and p.published = true;
"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<WhereColumnMatch>();

        let results: Vec<&WhereColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].get_column(sql), "email");
        assert_eq!(results[0].get_alias(sql), Some("u".into()));

        assert_eq!(results[1].get_column(sql), "published");
        assert_eq!(results[1].get_alias(sql), Some("p".into()));
    }

    #[test]
    fn finds_columns_in_complex_where_clause() {
        let sql = r#"
select * from users u
where u.active = true and (u.role = 'admin' or u.role = 'moderator');
"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<WhereColumnMatch>();

        let results: Vec<&WhereColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert!(results.len() == 3);
        assert_eq!(results[0].get_column(sql), "active");
        assert_eq!(results[0].get_alias(sql), Some("u".into()));

        assert_eq!(results[1].get_column(sql), "role");
        assert_eq!(results[1].get_alias(sql), Some("u".into()));

        assert_eq!(results[2].get_column(sql), "role");
        assert_eq!(results[2].get_alias(sql), Some("u".into()));
    }
}
