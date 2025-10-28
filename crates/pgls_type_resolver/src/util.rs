pub(crate) fn get_string_from_node(node: &pgls_query::protobuf::Node) -> String {
    match &node.node {
        Some(pgls_query::NodeEnum::String(s)) => s.sval.to_string(),
        _ => "".to_string(),
    }
}
