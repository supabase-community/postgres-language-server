use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;

/// An experimental API which parses a PLPGSQL function. This currently drops the returned
/// structure and returns only a Result<()>.
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
    let input = CString::new(stmt)?;
    let result = unsafe { pg_query_parse_plpgsql(input.as_ptr()) };
    let structure = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else {
        Ok(())
    };
    unsafe { pg_query_free_plpgsql_parse_result(result) };
    structure
}
