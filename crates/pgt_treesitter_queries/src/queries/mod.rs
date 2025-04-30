mod relations;
mod table_aliases;

pub use relations::*;
pub use table_aliases::*;

#[derive(Debug)]
pub enum QueryResult<'a> {
    Relation(RelationMatch<'a>),
    TableAliases(TableAliasMatch<'a>),
}

impl QueryResult<'_> {
    pub fn within_range(&self, range: &tree_sitter::Range) -> bool {
        match self {
            QueryResult::Relation(rm) => {
                let start = match rm.schema {
                    Some(s) => s.start_position(),
                    None => rm.table.start_position(),
                };

                let end = rm.table.end_position();

                start >= range.start_point && end <= range.end_point
            }
            QueryResult::TableAliases(m) => {
                let start = m.table.start_position();
                let end = m.alias.end_position();
                start >= range.start_point && end <= range.end_point
            }
        }
    }
}

// This trait enforces that for any `Self` that implements `Query`,
// its &Self must implement TryFrom<&QueryResult>
pub(crate) trait QueryTryFrom<'a>: Sized {
    type Ref: for<'any> TryFrom<&'a QueryResult<'a>, Error = String>;
}

pub(crate) trait Query<'a>: QueryTryFrom<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<crate::QueryResult<'a>>;
}
