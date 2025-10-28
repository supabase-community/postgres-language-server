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
        ) 
        (keyword_as)?
        (any_identifier) @alias
    )
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
            if m.captures.len() == 3 {
                let schema = m.captures[0].node;
                let table = m.captures[1].node;
                let alias = m.captures[2].node;

                to_return.push(QueryResult::TableAliases(TableAliasMatch {
                    table,
                    alias,
                    schema: Some(schema),
                }));
            }

            if m.captures.len() == 2 {
                let table = m.captures[0].node;
                let alias = m.captures[1].node;

                to_return.push(QueryResult::TableAliases(TableAliasMatch {
                    table,
                    alias,
                    schema: None,
                }));
            }
        });

        to_return
    }
}
