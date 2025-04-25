use std::sync::LazyLock;

use crate::{Query, QueryResult};

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
[
  (field
    (identifier)) @reference
  (field
    (object_reference)
    "." (identifier)) @reference
  (parameter) @parameter
]
"#;
    tree_sitter::Query::new(tree_sitter_sql::language(), QUERY_STR).expect("Invalid TS Query")
});

#[derive(Debug)]
pub struct ParameterMatch<'a> {
    pub(crate) root: Option<tree_sitter::Node<'a>>,
    pub(crate) path: Option<tree_sitter::Node<'a>>,

    pub(crate) field: tree_sitter::Node<'a>,
}

#[derive(Debug, PartialEq)]
pub enum Field {
    Text(String),
    Parameter(usize),
}

impl ParameterMatch<'_> {
    pub fn get_root(&self, sql: &str) -> Option<String> {
        let str = self
            .root
            .as_ref()?
            .utf8_text(sql.as_bytes())
            .expect("Failed to get schema from RelationMatch");

        Some(str.to_string())
    }

    pub fn get_path(&self, sql: &str) -> Option<String> {
        let str = self
            .path
            .as_ref()?
            .utf8_text(sql.as_bytes())
            .expect("Failed to get table from RelationMatch");

        Some(str.to_string())
    }

    pub fn get_field(&self, sql: &str) -> Field {
        let text = self
            .field
            .utf8_text(sql.as_bytes())
            .expect("Failed to get field from RelationMatch");

        if let Some(stripped) = text.strip_prefix('$') {
            return Field::Parameter(
                stripped
                    .parse::<usize>()
                    .expect("Failed to parse parameter"),
            );
        }

        Field::Text(text.to_string())
    }

    pub fn get_range(&self) -> tree_sitter::Range {
        self.field.range()
    }

    pub fn get_byte_range(&self) -> std::ops::Range<usize> {
        let range = self.field.range();
        range.start_byte..range.end_byte
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a ParameterMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::Parameter(r) => Ok(r),

            #[allow(unreachable_patterns)]
            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for ParameterMatch<'a> {
    type Ref = &'a ParameterMatch<'a>;
}

impl<'a> Query<'a> for ParameterMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<crate::QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        matches
            .filter_map(|m| {
                let captures = m.captures;

                // We expect exactly one capture for a parameter
                if captures.len() != 1 {
                    return None;
                }

                let field = captures[0].node;
                let text = match field.utf8_text(stmt.as_bytes()) {
                    Ok(t) => t,
                    Err(_) => return None,
                };
                let parts: Vec<&str> = text.split('.').collect();

                let param_match = match parts.len() {
                    // Simple field: field_name
                    1 => ParameterMatch {
                        root: None,
                        path: None,
                        field,
                    },
                    // Table qualified: table.field_name
                    2 => ParameterMatch {
                        root: None,
                        path: field.named_child(0),
                        field: field.named_child(1)?,
                    },
                    // Fully qualified: schema.table.field_name
                    3 => ParameterMatch {
                        root: field.named_child(0).and_then(|n| n.named_child(0)),
                        path: field.named_child(0).and_then(|n| n.named_child(1)),
                        field: field.named_child(1)?,
                    },
                    // Unexpected number of parts
                    _ => return None,
                };

                Some(QueryResult::Parameter(param_match))
            })
            .collect()
    }
}
