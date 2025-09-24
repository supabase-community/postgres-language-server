use protobuf::Node;

pgls_query_macros::node_mut_codegen!();

impl NodeMut {
    pub fn deparse(&self) -> Result<std::string::String> {
        crate::deparse(&protobuf::ParseResult {
            version: crate::bindings::PG_VERSION_NUM as i32,
            stmts: vec![protobuf::RawStmt {
                stmt: Some(Box::new(Node {
                    node: Some(self.to_enum()?),
                })),
                stmt_location: 0,
                stmt_len: 0,
            }],
        })
    }

    pub fn nodes_mut(&self) -> Vec<NodeMut> {
        self.iter_mut().collect()
    }

    pub fn iter_mut(&self) -> NodeMutIterator {
        NodeMutIterator::new(*self)
    }
}
