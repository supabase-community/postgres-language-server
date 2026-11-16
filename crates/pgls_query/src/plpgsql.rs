use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;

/// An experimental API which parses a PLPGSQL function. This drops the returned structure and
/// returns only a Result<()>, see [`parse_plpgsql_json`] for the structure.
///
/// # Example
///
/// ```rust
/// let result = pgls_query::parse_plpgsql("
///     CREATE OR REPLACE FUNCTION cs_fmt_browser_version(v_name varchar, v_version varchar)
///     RETURNS varchar AS $$
///     BEGIN
///         IF v_version IS NULL THEN
///             RETURN v_name;
///         END IF;
///         RETURN v_name || '/' || v_version;
///     END;
///     $$ LANGUAGE plpgsql;
/// ");
/// assert!(result.is_ok());
/// ```
pub fn parse_plpgsql(stmt: &str) -> Result<()> {
    parse_plpgsql_json(stmt).map(|_| ())
}

/// Parses a PLPGSQL function like [`parse_plpgsql`], and returns libpg_query's parse tree of the
/// function as JSON: an array with one `{"PLpgSQL_function": ...}` object.
///
/// # Example
///
/// ```rust
/// let json = pgls_query::parse_plpgsql_json("
///     CREATE FUNCTION f() RETURNS void AS $$
///     BEGIN
///         CREATE TABLE t (a int);
///     END;
///     $$ LANGUAGE plpgsql;
/// ")
/// .unwrap();
/// assert!(json.contains("CREATE TABLE t (a int)"));
/// ```
pub fn parse_plpgsql_json(stmt: &str) -> Result<String> {
    let input = CString::new(stmt)?;
    let result = unsafe { pg_query_parse_plpgsql(input.as_ptr()) };
    let structure = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else if result.plpgsql_funcs.is_null() {
        Err(Error::Parse("no PL/pgSQL function".to_string()))
    } else {
        Ok(unsafe { CStr::from_ptr(result.plpgsql_funcs) }
            .to_string_lossy()
            .into_owned())
    };
    unsafe { pg_query_free_plpgsql_parse_result(result) };
    structure
}
