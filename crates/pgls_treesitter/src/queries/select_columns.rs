use std::sync::LazyLock;

use crate::queries::{Query, QueryResult};
use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (select_expression
        (term
            (field
                (field_qualifier
                    (object_reference) @alias
                    "."
                )?
                (column_identifier) @column
            )
        )
        ","?
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
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
                .expect("Failed to get alias from ColumnMatch")
                .to_string(),
        )
    }

    pub fn get_column(&self, sql: &str) -> String {
        self.column
            .utf8_text(sql.as_bytes())
            .expect("Failed to get column from ColumnMatch")
            .to_string()
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a SelectColumnMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::SelectClauseColumns(c) => Ok(c),

            #[allow(unreachable_patterns)]
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

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            if m.captures.len() == 1 {
                let capture = m.captures[0].node;
                to_return.push(QueryResult::SelectClauseColumns(SelectColumnMatch {
                    alias: None,
                    column: capture,
                }));
            }

            if m.captures.len() == 2 {
                let alias = m.captures[0].node;
                let column = m.captures[1].node;

                to_return.push(QueryResult::SelectClauseColumns(SelectColumnMatch {
                    alias: Some(alias),
                    column,
                }));
            }
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
    cs.client_id
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
    }
}
