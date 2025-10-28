use crate::*;

use protobuf::Node;
pub use protobuf::node::Node as NodeEnum;

pgls_query_macros::node_enum_codegen!();

impl NodeEnum {
    pub fn deparse(&self) -> Result<std::string::String> {
        crate::deparse(&protobuf::ParseResult {
            version: crate::bindings::PG_VERSION_NUM as i32,
            stmts: vec![protobuf::RawStmt {
                stmt: Some(Box::new(Node {
                    node: Some(self.clone()),
                })),
                stmt_location: 0,
                stmt_len: 0,
            }],
        })
    }

    pub fn nodes(&self) -> Vec<NodeRef<'_>> {
        self.iter().collect()
    }

    pub fn iter(&self) -> NodeRefIterator<'_> {
        NodeRefIterator::new(self.to_ref())
    }

    pub fn iter_mut(&mut self) -> NodeMutIterator {
        NodeMutIterator::new(self.to_mut())
    }
}
