use std::sync::LazyLock;

use crate::queries::{Query, QueryResult, object_references::parts_of_reference_query};
use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    [
        (relation
            (table_reference) @ref
            (alias
                (keyword_as)?
                (any_identifier) @alias
            )?
        )
        ; The target of UPDATE and DELETE.
        (update_target
            (table_reference) @ref
            (alias
                (keyword_as)?
                (any_identifier) @alias
            )
        )
        (delete_statement
            (from
                (table_reference) @ref
                (alias
                    (keyword_as)?
                    (any_identifier) @alias
                )
            )
        )
        ; The target and the source of MERGE.
        (merge
            (table_reference) @ref
            .
            (alias
                (keyword_as)?
                (any_identifier) @alias
            )
        )
    ]
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

#[derive(Debug)]
pub struct TableAliasMatch<'a> {
    pub(crate) table: tree_sitter::Node<'a>,
    pub(crate) alias: tree_sitter::Node<'a>,
    pub(crate) schema: Option<tree_sitter::Node<'a>>,
}

impl TableAliasMatch<'_> {
    pub fn get_alias(&self, sql: &str) -> String {
        self.alias
            .utf8_text(sql.as_bytes())
            .expect("Failed to get alias from TableAliasMatch")
            .to_string()
    }

    pub fn get_table(&self, sql: &str) -> String {
        self.table
            .utf8_text(sql.as_bytes())
            .expect("Failed to get table from TableAliasMatch")
            .to_string()
    }

    pub fn get_schema(&self, sql: &str) -> Option<String> {
        self.schema.as_ref().map(|n| {
            n.utf8_text(sql.as_bytes())
                .expect("Failed to get table from TableAliasMatch")
                .to_string()
        })
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a TableAliasMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::TableAliases(t) => Ok(t),

            #[allow(unreachable_patterns)]
            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for TableAliasMatch<'a> {
    type Ref = &'a TableAliasMatch<'a>;
}

impl<'a> Query<'a> for TableAliasMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            if m.captures.len() == 2 {
                let obj_ref = m.captures[0].node;
                let alias = m.captures[1].node;
                if let Some((_, schema, table)) = parts_of_reference_query(obj_ref, stmt) {
                    to_return.push(QueryResult::TableAliases(TableAliasMatch {
                        schema,
                        table,
                        alias,
                    }));
                }
            }
        });

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::TableAliasMatch;

    #[test]
    fn finds_table_alias_without_schema() {
        let sql = r#"select id from users u;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<&TableAliasMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_table(sql), "users");
        assert_eq!(results[0].get_alias(sql), "u");
        assert_eq!(results[0].get_schema(sql), None);
    }

    #[test]
    fn finds_update_and_delete_target_aliases() {
        let sql = r#"
update auth.users u set email = 'x' from posts as p where p.user_id = u.id;
delete from auth.users as d using comments c where c.user_id = d.id;
"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<(String, String)> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .map(|m: &TableAliasMatch| (m.get_alias(sql), m.get_table(sql)))
            .collect();

        assert_eq!(
            results,
            [
                ("u".to_string(), "users".to_string()),
                ("p".to_string(), "posts".to_string()),
                ("d".to_string(), "users".to_string()),
                ("c".to_string(), "comments".to_string()),
            ]
        );
    }

    #[test]
    fn finds_table_alias_with_schema() {
        let sql = r#"select id from auth.users u;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<&TableAliasMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_table(sql), "users");
        assert_eq!(results[0].get_alias(sql), "u");
        assert_eq!(results[0].get_schema(sql), Some("auth".into()));
    }

    #[test]
    fn finds_multiple_table_aliases() {
        let sql = r#"
select
    u.id,
    u.email,
    cs.user_settings
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

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<&TableAliasMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].get_table(sql), "users");
        assert_eq!(results[0].get_alias(sql), "u");
        assert_eq!(results[0].get_schema(sql), Some("auth".into()));

        assert_eq!(results[1].get_table(sql), "client_settings");
        assert_eq!(results[1].get_alias(sql), "cs");
        assert_eq!(results[1].get_schema(sql), Some("public".into()));
    }

    #[test]
    fn finds_table_alias_with_as_keyword() {
        let sql = r#"select id from users as u;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<&TableAliasMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_table(sql), "users");
        assert_eq!(results[0].get_alias(sql), "u");
        assert_eq!(results[0].get_schema(sql), None);
    }

    #[test]
    fn finds_merge_aliases() {
        let sql = r#"merge into auth.users as u using posts p on p.user_id = u.id when matched then delete;"#;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<TableAliasMatch>();

        let results: Vec<(String, String)> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .map(|m: &TableAliasMatch| (m.get_table(sql), m.get_alias(sql)))
            .collect();

        assert_eq!(
            results,
            [
                ("users".to_string(), "u".to_string()),
                ("posts".to_string(), "p".to_string())
            ]
        );
    }
}
