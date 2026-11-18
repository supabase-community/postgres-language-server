use std::sync::LazyLock;

use crate::queries::{Query, QueryResult, object_references::parts_of_reference_query};
use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (select_expression
        (term
            (object_reference) @ref
        )
        ","?
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

#[derive(Debug)]
pub struct SelectColumnMatch<'a> {
    pub(crate) alias: Option<tree_sitter::Node<'a>>,
    pub(crate) column: tree_sitter::Node<'a>,
}

impl SelectColumnMatch<'_> {
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

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a SelectColumnMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::SelectClauseColumns(c) => Ok(c),

            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for SelectColumnMatch<'a> {
    type Ref = &'a SelectColumnMatch<'a>;
}

impl<'a> Query<'a> for SelectColumnMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let Some(query) = TS_QUERY.as_ref() else {
            return vec![];
        };
        let matches = cursor.matches(query, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            m.captures.iter().for_each(|capture| {
                if let Some((_, alias, column)) = parts_of_reference_query(capture.node, stmt) {
                    to_return.push(QueryResult::SelectClauseColumns(SelectColumnMatch {
                        alias,
                        column,
                    }));
                }
            });
        });

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::SelectColumnMatch;

    #[test]
    fn finds_all_columns() {
        let sql = r#"select aud, id, email from auth.users;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<SelectColumnMatch>();

        let results: Vec<&SelectColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results[0].get_alias(sql), None);
        assert_eq!(results[0].get_column(sql), "aud");

        assert_eq!(results[1].get_alias(sql), None);
        assert_eq!(results[1].get_column(sql), "id");

        assert_eq!(results[2].get_alias(sql), None);
        assert_eq!(results[2].get_column(sql), "email");
    }

    #[test]
    fn finds_columns_with_aliases() {
        let sql = r#"
select 
    u.id,
    u.email,
    cs.user_settings,
    cs.client_id,
    public.client_settings.id
from 
    auth.users u
    join public.client_settings cs
    on u.id = cs.user_id;

"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<SelectColumnMatch>();

        let results: Vec<&SelectColumnMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results[0].get_alias(sql), Some("u".into()));
        assert_eq!(results[0].get_column(sql), "id");

        assert_eq!(results[1].get_alias(sql), Some("u".into()));
        assert_eq!(results[1].get_column(sql), "email");

        assert_eq!(results[2].get_alias(sql), Some("cs".into()));
        assert_eq!(results[2].get_column(sql), "user_settings");

        assert_eq!(results[3].get_alias(sql), Some("cs".into()));
        assert_eq!(results[3].get_column(sql), "client_id");

        assert_eq!(results[4].get_alias(sql), Some("client_settings".into()));
        assert_eq!(results[4].get_column(sql), "id");
    }
}
