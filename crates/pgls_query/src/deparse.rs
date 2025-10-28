use std::ffi::CStr;
use std::os::raw::c_char;

use crate::bindings::*;
use crate::error::*;
use crate::protobuf;

use prost::Message;

/// Converts a parsed tree back into a string.
///
/// # Example
///
/// ```rust
/// use pgls_query::{parse, NodeEnum, NodeRef};
///
/// let result = parse("INSERT INTO other (name) SELECT name FROM contacts");
/// let result = result.unwrap();
/// let stmts = result.stmts();
/// let insert = stmts.first().unwrap();
/// assert!(matches!(insert, NodeEnum::InsertStmt(_)));
/// let select = insert.iter().find(|n| matches!(n, NodeRef::SelectStmt(_))).unwrap();
///
/// // The entire parse result can be deparsed:
/// assert_eq!(result.deparse().unwrap(), "INSERT INTO other (name) SELECT name FROM contacts");
/// // Or an individual node can be deparsed:
/// assert_eq!(insert.deparse().unwrap(), "INSERT INTO other (name) SELECT name FROM contacts");
/// assert_eq!(select.deparse().unwrap(), "SELECT name FROM contacts");
/// ```
///
/// Note that this function will panic if called on a node not defined in `deparseStmt`
pub fn deparse(protobuf: &protobuf::ParseResult) -> Result<String> {
    let buffer = protobuf.encode_to_vec();
    let len = buffer.len();
    let data = buffer.as_ptr() as *const c_char as *mut c_char;
    let protobuf = PgQueryProtobuf { data, len };
    let result = unsafe { pg_query_deparse_protobuf(protobuf) };

    let deparse_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else {
        let query = unsafe { CStr::from_ptr(result.query) }
            .to_string_lossy()
            .to_string();
        Ok(query)
    };

    unsafe { pg_query_free_deparse_result(result) };
    deparse_result
}

#[cfg(test)]
mod tests {
    use crate::parse;

    fn assert_deparse(input: &str, output: &str) {
        let result = parse(input).unwrap();
        assert_eq!(result.deparse().unwrap(), output);
    }

    #[test]
    fn it_deparses_select() {
        let query = "SELECT a AS b FROM x WHERE y = 5 AND z = y";
        assert_deparse(query, query);
    }

    #[test]
    fn it_deparses_select_with_empty_target_list() {
        let query = "SELECT FROM x WHERE y = 5 AND z = y";
        assert_deparse(query, query);
    }

    #[test]
    fn it_deparses_select_with_schema() {
        let query = "SELECT a AS b FROM public.x WHERE y = 5 AND z = y";
        assert_deparse(query, query);
    }

    #[test]
    fn it_deparses_select_with_distinct() {
        let query = "SELECT DISTINCT a, b, * FROM c WHERE d = e";
        assert_deparse(query, query);
    }

    #[test]
    fn it_deparses_select_with_distinct_on() {
        let query = "SELECT DISTINCT ON (a) a, b FROM c";
        assert_deparse(query, query);
    }
}
