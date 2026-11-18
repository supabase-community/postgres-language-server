use std::sync::LazyLock;

use crate::queries::{Query, QueryResult};

use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
[
  (object_reference) @reference

  (parameter) @parameter
]
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

#[derive(Debug)]
pub struct ParameterMatch<'a> {
    pub(crate) node: tree_sitter::Node<'a>,
}

impl ParameterMatch<'_> {
    pub fn get_path(&self, sql: &str) -> String {
        self.node
            .utf8_text(sql.as_bytes())
            // A node of `sql` is UTF-8, unless the tree is stale.
            .unwrap_or_default()
            .to_string()
    }

    pub fn get_range(&self) -> tree_sitter::Range {
        self.node.range()
    }

    pub fn get_byte_range(&self) -> std::ops::Range<usize> {
        let range = self.node.range();
        range.start_byte..range.end_byte
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a ParameterMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::Parameter(r) => Ok(r),

            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for ParameterMatch<'a> {
    type Ref = &'a ParameterMatch<'a>;
}

impl<'a> Query<'a> for ParameterMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let Some(query) = TS_QUERY.as_ref() else {
            return vec![];
        };
        let matches = cursor.matches(query, root_node, stmt.as_bytes());

        let mut result = vec![];

        matches.for_each(|m| {
            let captures = m.captures;

            // We expect exactly one capture for a parameter
            if let [capture] = captures {
                result.push(QueryResult::Parameter(ParameterMatch {
                    node: capture.node,
                }))
            }
        });

        result
    }
}
