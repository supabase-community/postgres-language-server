use std::sync::LazyLock;

use crate::queries::{Query, QueryResult};

use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    (where
        (binary_expression
            (binary_expression 
                (field
                    (field_qualifier
                        (object_reference) @alias
                        "."
                    )?
                    (column_identifier) @column
                )
            )
        )
    )
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
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

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a WhereColumnMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::WhereClauseColumns(c) => Ok(c),

            #[allow(unreachable_patterns)]
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

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let mut to_return = vec![];

        matches.for_each(|m| {
            if m.captures.len() == 1 {
                let capture = m.captures[0].node;
                to_return.push(QueryResult::WhereClauseColumns(WhereColumnMatch {
                    alias: None,
                    column: capture,
                }));
            }

            if m.captures.len() == 2 {
                let alias = m.captures[0].node;
                let column = m.captures[1].node;

                to_return.push(QueryResult::WhereClauseColumns(WhereColumnMatch {
                    alias: Some(alias),
                    column,
                }));
            }
        });

        to_return
    }
}
