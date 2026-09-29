use pgls_query::{Node, NodeEnum};

/// The value of a `String` node.
pub(super) fn string_value(node: &Node) -> Option<&str> {
    match node.node.as_ref()? {
        NodeEnum::String(value) => Some(&value.sval),
        _ => None,
    }
}

/// All values of a list of `String` nodes. `None` if a node is not a string.
pub(super) fn string_values(nodes: &[Node]) -> Option<Vec<String>> {
    nodes
        .iter()
        .map(|node| string_value(node).map(str::to_owned))
        .collect()
}
