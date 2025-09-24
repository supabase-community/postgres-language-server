use protobuf::Node;

pgls_query_macros::node_structs_codegen!();

impl Node {
    pub fn deparse(&self) -> Result<std::string::String> {
        crate::deparse(&protobuf::ParseResult {
            version: crate::bindings::PG_VERSION_NUM as i32,
            stmts: vec![protobuf::RawStmt {
                stmt: Some(Box::new(self.clone())),
                stmt_location: 0,
                stmt_len: 0,
            }],
        })
    }
}
