use crate::{
    NodeEnum,
    protobuf::{Constraint, Node},
};

impl Constraint {
    /// The constraint nodes of a list, e.g. the constraints of a `CREATE DOMAIN` or of a column
    /// definition. `DEFAULT`, `NULL`, `NOT NULL` and `GENERATED` are constraint nodes too.
    pub fn in_list(nodes: &[Node]) -> impl Iterator<Item = &Constraint> {
        nodes.iter().filter_map(|node| match &node.node {
            Some(NodeEnum::Constraint(constraint)) => Some(constraint.as_ref()),
            _ => None,
        })
    }
}
