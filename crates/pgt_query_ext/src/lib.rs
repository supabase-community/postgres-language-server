//! Postgres Statement Parser
//!
//! Simple wrapper crate for `pg_query` to expose types and a function to get the root node for an
//! SQL statement.
//!
//! It also host any "extensions" to the `pg_query` crate that are not yet contributed upstream.
//! Extensions include
//! - `get_location` to get the location of a node
//! - `get_node_properties` to get the properties of a node
//! - `get_nodes` to get all the nodes in the AST as a petgraph tree
//! - `ChildrenIterator` to iterate over the children of a node
mod codegen;
pub mod diagnostics;

pub use pg_query::protobuf;
pub use pg_query::{Error, NodeEnum, Result};

pub use codegen::{
    ChildrenIterator, Node, TokenProperty, get_location, get_node_properties, get_nodes,
};

pub fn parse(sql: &str) -> Result<NodeEnum> {
    pg_query::parse(sql).map(|parsed| {
        parsed
            .protobuf
            .nodes()
            .iter()
            .find(|n| n.1 == 1)
            .map(|n| n.0.to_enum())
            .ok_or_else(|| Error::Parse("Unable to find root node".to_string()))
    })?
}
