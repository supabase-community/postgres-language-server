pub mod group_kind;
pub mod node_location;
pub mod token_kind;

#[cfg(test)]
mod tests {
    use crate::codegen::node_location::node_location;
    use pgls_query::NodeEnum;

    #[test]
    fn a_node_carrying_a_location_reports_it() {
        let parsed = pgls_query::parse("SELECT 1 FROM s.t").expect("parse");
        let ast = parsed.into_root().expect("root");

        let located = ast
            .iter()
            .filter(|node| node_location(node).is_some())
            .count();
        assert!(located > 0, "a select statement has located nodes");
    }

    #[test]
    fn a_node_without_a_location_reports_none() {
        let node = NodeEnum::Boolean(pgls_query::protobuf::Boolean { boolval: true });
        assert_eq!(node_location(&node.to_ref()), None);
    }
}
