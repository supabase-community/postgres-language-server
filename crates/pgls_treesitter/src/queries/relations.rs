use std::sync::LazyLock;

use crate::queries::{Query, QueryResult};

use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (relation
        (object_reference
            .
            (any_identifier) @schema_or_table
            "."?
            (any_identifier)? @table
        )+
    )
    (insert
        (object_reference
            .
            (any_identifier) @schema_or_table
            "."?
            (any_identifier)? @table
        )+
    )
    (alter_table
        (keyword_alter)
        (keyword_table)
        (object_reference
            .
            (any_identifier) @schema_or_table
            "."?
            (any_identifier)? @table
        )+
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

#[derive(Debug)]
pub struct RelationMatch<'a> {
    pub(crate) schema: Option<tree_sitter::Node<'a>>,
    pub(crate) table: tree_sitter::Node<'a>,
}

impl RelationMatch<'_> {
    pub fn get_schema(&self, sql: &str) -> Option<String> {
        Some(
            self.schema
                .as_ref()?
                .utf8_text(sql.as_bytes())
                .expect("Failed to get schema from RelationMatch")
                .to_string(),
        )
    }

    pub fn get_table(&self, sql: &str) -> String {
        self.table
            .utf8_text(sql.as_bytes())
            .expect("Failed to get table from RelationMatch")
            .to_string()
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a RelationMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::Relation(r) => Ok(r),

            #[allow(unreachable_patterns)]
            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for RelationMatch<'a> {
    type Ref = &'a RelationMatch<'a>;
}

impl<'a> Query<'a> for RelationMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            if m.captures.len() == 1 {
                let capture = m.captures[0].node;
                to_return.push(QueryResult::Relation(RelationMatch {
                    schema: None,
                    table: capture,
                }));
            }

            if m.captures.len() == 2 {
                let schema = m.captures[0].node;
                let table = m.captures[1].node;

                to_return.push(QueryResult::Relation(RelationMatch {
                    schema: Some(schema),
                    table,
                }));
            }
        });

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::RelationMatch;

    #[test]
    fn finds_table_without_schema() {
        let sql = r#"select * from users;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), None);
        assert_eq!(results[0].get_table(sql), "users");
    }

    #[test]
    fn finds_table_with_schema() {
        let sql = r#"select * from public.users;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), Some("public".to_string()));
        assert_eq!(results[0].get_table(sql), "users");
    }

    #[test]
    fn finds_table_with_schema_quotes() {
        let sql = r#"select * from "public"."users";"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), Some(r#""public""#.to_string()));
        assert_eq!(results[0].get_table(sql), r#""users""#);
    }

    #[test]
    fn finds_insert_into_with_schema_and_table() {
        let sql = r#"insert into auth.accounts (id, email) values (1, 'a@b.com');"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), Some("auth".to_string()));
        assert_eq!(results[0].get_table(sql), "accounts");
    }

    #[test]
    fn finds_insert_into_without_schema() {
        let sql = r#"insert into users (id, email) values (1, 'a@b.com');"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), None);
        assert_eq!(results[0].get_table(sql), "users");
    }

    #[test]
    fn finds_alter_table_with_schema() {
        let sql = r#"alter table public.users alter some_col set default 15;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), Some("public".into()));
        assert_eq!(results[0].get_table(sql), "users");
    }

    #[test]
    fn finds_alter_table_without_schema() {
        let sql = r#"alter table users alter some_col set default 15;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), None);
        assert_eq!(results[0].get_table(sql), "users");
    }
}
