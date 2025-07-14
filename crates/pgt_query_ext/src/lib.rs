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
pub mod diagnostics;

pub use pg_query::protobuf;
pub use pg_query::{Error, NodeEnum, Result};

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

/// This function parses a PL/pgSQL function.
///
/// It expects the entire `CREATE FUNCTION` statement.
pub fn parse_plpgsql(sql: &str) -> Result<()> {
    // we swallow the error until we have a proper binding
    let _ = pg_query::parse_plpgsql(sql)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plpgsql_err() {
        let input = "
create function test_organisation_id ()
    returns setof text
    language plpgsql
    security invoker
    as $$
    -- syntax error here
    decare
        v_organisation_id uuid;
begin
    select 1;
end
$$;
        ";

        assert!(parse_plpgsql(input).is_err());
    }
}
