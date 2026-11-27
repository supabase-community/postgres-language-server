use crate::{NodeEnum, protobuf::Node};

impl Node {
    /// The value of a `String` node.
    pub fn string_value(&self) -> Option<&str> {
        match self.node.as_ref()? {
            NodeEnum::String(value) => Some(&value.sval),
            _ => None,
        }
    }

    /// The values of a list of `String` nodes, e.g. `["public", "users"]`. `None` if a node is not
    /// a string.
    pub fn string_values(nodes: &[Node]) -> Option<Vec<String>> {
        nodes
            .iter()
            .map(|node| node.string_value().map(str::to_owned))
            .collect()
    }
}
