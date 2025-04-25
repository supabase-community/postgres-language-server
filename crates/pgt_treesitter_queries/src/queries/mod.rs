mod parameters;
mod relations;

pub use parameters::*;
pub use relations::*;

#[derive(Debug)]
pub enum QueryResult<'a> {
    Relation(RelationMatch<'a>),
    Parameter(ParameterMatch<'a>),
}

impl QueryResult<'_> {
    pub fn within_range(&self, range: &tree_sitter::Range) -> bool {
        match self {
            Self::Relation(rm) => {
                let start = match rm.schema {
                    Some(s) => s.start_position(),
                    None => rm.table.start_position(),
                };

                let end = rm.table.end_position();

                start >= range.start_point && end <= range.end_point
            }
            Self::Parameter(pm) => {
                let start = match pm.root {
                    Some(s) => s.start_position(),
                    None => pm.path.as_ref().unwrap().start_position(),
                };

                let end = pm.field.end_position();

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
