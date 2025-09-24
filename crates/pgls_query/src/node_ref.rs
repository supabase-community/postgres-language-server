use protobuf::Node;

pgls_query_macros::node_ref_codegen!();

impl<'a> NodeRef<'a> {
    pub fn deparse(&self) -> Result<std::string::String> {
        crate::deparse(&protobuf::ParseResult {
            version: crate::bindings::PG_VERSION_NUM as i32,
            stmts: vec![protobuf::RawStmt {
                stmt: Some(Box::new(Node {
                    node: Some(self.to_enum()),
                })),
                stmt_location: 0,
                stmt_len: 0,
            }],
        })
    }

    pub fn nodes(&self) -> Vec<NodeRef<'a>> {
        self.iter().collect()
    }

    pub fn iter(&self) -> NodeRefIterator<'a> {
        NodeRefIterator::new(*self)
    }
}
