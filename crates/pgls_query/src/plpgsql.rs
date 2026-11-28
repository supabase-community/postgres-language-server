use std::ffi::{CStr, CString};
use std::sync::LazyLock;

use regex::Regex;

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

/// Messages libpg_query reports on valid PL/pgSQL functions because it compiles them without a
/// catalog: it only knows the types of `pg_catalog`, reads an unknown type in `public` as
/// `record`, so an array of one as `_record`, which a variable can't have, finds no element type
/// for a `VARIADIC` array, and fails with `Not implemented` on anything else, e.g. a type in
/// another schema. Postgres writes an array of `record` as `record[]`, so a message with
/// `_record` only comes from libpg_query.
/// https://github.com/pganalyze/libpg_query/blob/18.1.0/src/postgres/src_backend_utils_cache_syscache.c#L332-L344
const NO_CATALOG_ERRORS: &[&str] = &[
    "Not implemented",
    "cannot accept type _record",
    "cannot return type _record",
    "has pseudo-type _record",
    "VARIADIC parameter must be an array",
];

/// `"v_row.id" is not a known variable`: without a catalog, libpg_query doesn't know the fields
/// of a composite type. https://github.com/pganalyze/libpg_query/issues/159
#[expect(
    clippy::unwrap_used,
    reason = "the pattern is a constant, which `recognises_composite_type_errors` compiles"
)]
static COMPOSITE_TYPE_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\\?"([^"\\]+\.[^"\\]+)\\?" is not a known variable"#).unwrap());

impl Error {
    /// Whether [`parse_plpgsql`] reports this error on a function that may be valid, because
    /// libpg_query compiles it without a catalog: it doesn't know the fields of a composite type
    /// or any type outside `pg_catalog`.
    pub fn is_false_positive_without_catalog(&self) -> bool {
        match self {
            Error::Parse(message) => {
                COMPOSITE_TYPE_ERROR_RE.is_match(message)
                    || NO_CATALOG_ERRORS
                        .iter()
                        .any(|error| message.contains(error))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_composite_type_errors() {
        assert!(
            Error::Parse(r#""v_row.id" is not a known variable"#.to_string())
                .is_false_positive_without_catalog()
        );
        assert!(
            !Error::Parse(r#""v_id" is not a known variable"#.to_string())
                .is_false_positive_without_catalog()
        );
    }

    #[test]
    fn recognises_errors_on_functions_valid_without_a_catalog() {
        for input in [
            "create function f(x my_enum[]) returns void language plpgsql as $$ begin end $$",
            "create function f() returns my_enum[] language plpgsql as $$ begin return null; end $$",
            "create function f() returns void language plpgsql as $$ declare u auth.users; begin end $$",
            "create function f(variadic b int[]) returns void language plpgsql as $$ begin end $$",
            "create function f() returns void language plpgsql as $$ declare x my_type[]; begin end $$",
        ] {
            let err = parse_plpgsql(input).expect_err(input);
            assert!(err.is_false_positive_without_catalog(), "{input}: {err}");
        }
    }
}
