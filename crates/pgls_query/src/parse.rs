use std::ffi::{CStr, CString};

use crate::NodeEnum;
use crate::bindings::*;
use crate::error::*;
use crate::protobuf;

use prost::Message;

/// Parses the given SQL statement into the given abstract syntax tree.
///
/// # Example
///
/// ```rust
/// use pgls_query::parse;
///
/// let result = parse("SELECT * FROM contacts");
/// assert!(result.is_ok());
/// let result = result.unwrap();
/// assert_eq!(result.protobuf.stmts.len(), 1);
/// ```
pub fn parse(statement: &str) -> Result<ParseResult> {
    let input = CString::new(statement)?;
    let result = unsafe { pg_query_parse_protobuf(input.as_ptr()) };
    let parse_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else {
        let data = unsafe {
            std::slice::from_raw_parts(
                result.parse_tree.data as *const u8,
                result.parse_tree.len as usize,
            )
        };
        let stderr = unsafe { CStr::from_ptr(result.stderr_buffer) }
            .to_string_lossy()
            .to_string();
        protobuf::ParseResult::decode(data)
            .map_err(Error::Decode)
            .map(|result| ParseResult::new(result, stderr))
    };
    unsafe { pg_query_free_protobuf_parse_result(result) };
    parse_result
}

/// The result of parsing a SQL query
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed protobuf result
    pub protobuf: protobuf::ParseResult,
    /// Warnings captured during parsing
    pub warnings: Vec<String>,
}

impl ParseResult {
    /// Create a new ParseResult
    pub fn new(protobuf: protobuf::ParseResult, stderr: String) -> Self {
        let warnings = stderr
            .lines()
            .filter_map(|l| {
                if l.starts_with("WARNING") {
                    Some(l.trim().into())
                } else {
                    None
                }
            })
            .collect();

        Self { protobuf, warnings }
    }

    pub fn deparse(&self) -> Result<String> {
        crate::deparse(&self.protobuf)
    }

    pub fn stmts(&self) -> Vec<&NodeEnum> {
        self.protobuf
            .stmts
            .iter()
            .filter_map(|s| s.stmt.as_ref().and_then(|s| s.node.as_ref()))
            .collect()
    }

    pub fn stmts_mut(&mut self) -> Vec<&mut NodeEnum> {
        self.protobuf
            .stmts
            .iter_mut()
            .filter_map(|s| s.stmt.as_mut().and_then(|s| s.node.as_mut()))
            .collect()
    }

    /// Returns a reference to the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    pub fn root(&self) -> Option<&NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Get the first (and only) statement
        let raw_stmt = &self.protobuf.stmts[0];

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.as_ref().and_then(|stmt| stmt.node.as_ref())
    }

    /// Consumes the ParseResult and returns the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    /// This method avoids cloning by taking ownership of the ParseResult.
    pub fn into_root(self) -> Option<NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Extract the first (and only) statement by taking ownership
        let raw_stmt = self.protobuf.stmts.into_iter().next()?;

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.and_then(|stmt| stmt.node)
    }

    /// Returns a mutable reference to the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    pub fn root_mut(&mut self) -> Option<&mut NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Get the first (and only) statement
        let raw_stmt = &mut self.protobuf.stmts[0];

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.as_mut().and_then(|stmt| stmt.node.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn it_parses_parameter_queries() {
        assert!(parse("select $0 + $1 + $2 + $3 + $4 + $5").is_ok());
    }
}
