pub(crate) fn get_string_from_node(node: &pgt_query::protobuf::Node) -> String {
    match &node.node {
        Some(pgt_query::NodeEnum::String(s)) => s.sval.to_string(),
        _ => "".to_string(),
    }
}
