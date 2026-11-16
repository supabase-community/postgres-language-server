use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;

/// Split a well-formed query into separate statements.
///
/// # Example
///
/// ```rust
/// let query = r#"select /*;*/ 1; select "2;", (select 3);"#;
/// let statements = pgls_query::split_with_parser(query).unwrap();
/// assert_eq!(statements, vec!["select /*;*/ 1", r#"select "2;", (select 3)"#]);
/// ```
///
/// However, `split_with_parser` will fail on malformed statements
///
/// ```rust
/// let query = "select 1; this statement is not sql; select 2;";
/// let result = pgls_query::split_with_parser(query);
/// let err = r#"syntax error at or near "this""#;
/// assert_eq!(result, Err(pgls_query::Error::Split(err.to_string())));
/// ```
pub fn split_with_parser(query: &str) -> Result<Vec<&str>> {
    split_ranges_with_parser(query)?
        .into_iter()
        .map(|range| {
            // not sure the range'll hold up for non-utf8 charsets
            query.get(range).ok_or_else(|| {
                Error::Split("a statement does not start or end at a character boundary".into())
            })
        })
        .collect()
}

/// The byte ranges of the statements of a well-formed query, like [`split_with_parser`]: from the
/// first token of a statement to its `;`, without it, or to the end of the query for the last
/// statement without a `;`. Comments before a statement are not in its range, those before its
/// `;` are.
///
/// ```rust
/// let query = "select 1; -- one\nselect 2 ;";
/// let ranges = pgls_query::split_ranges_with_parser(query).unwrap();
/// assert_eq!(ranges, vec![0..8, 17..26]);
/// ```
pub fn split_ranges_with_parser(query: &str) -> Result<Vec<std::ops::Range<usize>>> {
    let input = CString::new(query)?;
    let result = unsafe { pg_query_split_with_parser(input.as_ptr()) };
    let split_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Split(message))
    } else {
        let n_stmts = result.n_stmts as usize;
        let mut ranges = Vec::with_capacity(n_stmts);
        for offset in 0..n_stmts {
            let split_stmt = unsafe { *result.stmts.add(offset).read() };
            let start = split_stmt.stmt_location as usize;
            ranges.push(start..start + split_stmt.stmt_len as usize);
        }
        Ok(ranges)
    };
    unsafe { pg_query_free_split_result(result) };
    split_result
}

/// Split a potentially-malformed query into separate statements. Note that
/// invalid tokens will be skipped
/// ```rust
/// let query = r#"select /*;*/ 1; asdf; select "2;", (select 3); asdf"#;
/// let statements = pgls_query::split_with_scanner(query).unwrap();
/// assert_eq!(statements, vec![
///     "select /*;*/ 1",
///     // skipped " asdf" since it was an invalid token
///     r#" select "2;", (select 3)"#,
/// ]);
/// ```
pub fn split_with_scanner(query: &str) -> Result<Vec<&str>> {
    let input = CString::new(query)?;
    let result = unsafe { pg_query_split_with_scanner(input.as_ptr()) };
    let split_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Split(message))
    } else {
        // don't use result.stderr_buffer since it appears unused unless
        // libpg_query is compiled with DEBUG defined.
        let n_stmts = result.n_stmts as usize;
        let mut start: usize;
        let mut end: usize;
        let mut statements = Vec::with_capacity(n_stmts);
        for offset in 0..n_stmts {
            let split_stmt = unsafe { *result.stmts.add(offset).read() };
            start = split_stmt.stmt_location as usize;
            // TODO: consider comparing the new value of start to the old value
            // of end to see if any region larger than a statement-separator got skipped
            end = start + split_stmt.stmt_len as usize;
            statements.push(&query[start..end]);
        }
        Ok(statements)
    };
    unsafe { pg_query_free_split_result(result) };
    split_result
}
