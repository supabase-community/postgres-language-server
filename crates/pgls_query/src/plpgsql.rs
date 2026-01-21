use std::ffi::{CStr, CString};

use prost::Message;

use crate::bindings::*;
use crate::error::*;
use crate::protobuf;

/// Parses the given PL/pgSQL function into an abstract syntax tree.
///
/// # Example
///
/// ```rust
/// use pgls_query::parse_plpgsql;
///
/// let result = parse_plpgsql("
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
/// let result = result.unwrap();
/// assert_eq!(result.functions().len(), 1);
/// ```
pub fn parse_plpgsql(stmt: &str) -> Result<PlpgsqlParseResult> {
    let input = CString::new(stmt)?;
    let result = unsafe { pg_query_parse_plpgsql_protobuf(input.as_ptr()) };
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
        protobuf::PLpgSqlParseResult::decode(data)
            .map_err(Error::Decode)
            .map(PlpgsqlParseResult::new)
    };
    unsafe { pg_query_free_plpgsql_protobuf_parse_result(result) };
    parse_result
}

/// The result of parsing a PL/pgSQL function
#[derive(Debug)]
pub struct PlpgsqlParseResult {
    /// The parsed protobuf result
    pub protobuf: protobuf::PLpgSqlParseResult,
}

impl PlpgsqlParseResult {
    /// Create a new PlpgsqlParseResult
    pub fn new(protobuf: protobuf::PLpgSqlParseResult) -> Self {
        Self { protobuf }
    }

    /// Returns a reference to the single parsed PL/pgSQL function.
    ///
    /// Returns `None` if there is not exactly one function in the parse result.
    /// Use `functions()` if you need to handle multiple functions.
    pub fn function(&self) -> Option<&protobuf::PLpgSqlFunction> {
        if self.protobuf.plpgsql_funcs.len() != 1 {
            return None;
        }
        self.protobuf.plpgsql_funcs.first()
    }

    /// Consumes the result and returns the single parsed function.
    ///
    /// Returns `None` if there is not exactly one function in the parse result.
    /// Use `into_functions()` if you need to handle multiple functions.
    pub fn into_function(self) -> Option<protobuf::PLpgSqlFunction> {
        if self.protobuf.plpgsql_funcs.len() != 1 {
            return None;
        }
        self.protobuf.plpgsql_funcs.into_iter().next()
    }

    /// Returns a reference to the list of parsed PL/pgSQL functions
    pub fn functions(&self) -> &[protobuf::PLpgSqlFunction] {
        &self.protobuf.plpgsql_funcs
    }

    /// Consumes the result and returns the list of parsed functions
    pub fn into_functions(self) -> Vec<protobuf::PLpgSqlFunction> {
        self.protobuf.plpgsql_funcs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protobuf::p_lpg_sql_stmt::Stmt;

    #[test]
    fn test_parse_plpgsql_simple() {
        let result = parse_plpgsql(
            "
            CREATE OR REPLACE FUNCTION test_func()
            RETURNS void AS $$
            BEGIN
                NULL;
            END;
            $$ LANGUAGE plpgsql;
        ",
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        // Use function() for single function access
        let func = result.function().expect("should have exactly one function");
        assert!(func.action.is_some());

        // The body should contain statements
        let action = func.action.as_ref().unwrap();
        assert!(!action.body.is_empty());
    }

    #[test]
    fn test_parse_plpgsql_with_assignment() {
        let result = parse_plpgsql(
            "
            CREATE OR REPLACE FUNCTION add_numbers(a int, b int)
            RETURNS int AS $$
            DECLARE
                result int;
            BEGIN
                result := a + b;
                RETURN result;
            END;
            $$ LANGUAGE plpgsql;
        ",
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.functions().len(), 1);

        let func = &result.functions()[0];
        let action = func.action.as_ref().unwrap();

        // Should have assignment and return statements
        assert!(action.body.len() >= 2);

        // First statement should be an assignment
        let first_stmt = &action.body[0];
        assert!(matches!(first_stmt.stmt, Some(Stmt::StmtAssign(_))));

        // Second statement should be a return
        let second_stmt = &action.body[1];
        assert!(matches!(second_stmt.stmt, Some(Stmt::StmtReturn(_))));

        // Verify the assignment expression contains the query
        if let Some(Stmt::StmtAssign(assign)) = &first_stmt.stmt {
            assert!(assign.expr.is_some());
            let expr = assign.expr.as_ref().unwrap();
            assert!(expr.query.contains("a + b"));
        }
    }

    #[test]
    fn test_parse_plpgsql_with_if() {
        let result = parse_plpgsql(
            "
            CREATE OR REPLACE FUNCTION cs_fmt_browser_version(v_name varchar, v_version varchar)
            RETURNS varchar AS $$
            BEGIN
                IF v_version IS NULL THEN
                    RETURN v_name;
                END IF;
                RETURN v_name || '/' || v_version;
            END;
            $$ LANGUAGE plpgsql;
        ",
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.functions().len(), 1);

        let func = &result.functions()[0];
        let action = func.action.as_ref().unwrap();

        // Should have IF and RETURN statements
        assert!(action.body.len() >= 2);

        // First statement should be IF
        let if_stmt = &action.body[0];
        assert!(matches!(if_stmt.stmt, Some(Stmt::StmtIf(_))));

        // Verify the IF statement structure
        if let Some(Stmt::StmtIf(if_node)) = &if_stmt.stmt {
            // Should have a condition
            assert!(if_node.cond.is_some());
            let cond = if_node.cond.as_ref().unwrap();
            assert!(cond.query.contains("v_version IS NULL"));

            // Should have a then_body with RETURN statement
            assert!(!if_node.then_body.is_empty());
            assert!(matches!(
                if_node.then_body[0].stmt,
                Some(Stmt::StmtReturn(_))
            ));
        }

        // Second statement should be RETURN
        let return_stmt = &action.body[1];
        assert!(matches!(return_stmt.stmt, Some(Stmt::StmtReturn(_))));
    }

    #[test]
    fn test_parse_plpgsql_with_loop() {
        let result = parse_plpgsql(
            "
            CREATE OR REPLACE FUNCTION count_down(n int)
            RETURNS void AS $$
            BEGIN
                WHILE n > 0 LOOP
                    n := n - 1;
                END LOOP;
            END;
            $$ LANGUAGE plpgsql;
        ",
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        let func = &result.functions()[0];
        let action = func.action.as_ref().unwrap();

        // First statement should be WHILE loop
        let while_stmt = &action.body[0];
        assert!(matches!(while_stmt.stmt, Some(Stmt::StmtWhile(_))));

        if let Some(Stmt::StmtWhile(while_node)) = &while_stmt.stmt {
            // Should have a condition
            assert!(while_node.cond.is_some());
            let cond = while_node.cond.as_ref().unwrap();
            assert!(cond.query.contains("n > 0"));

            // Should have a body with assignment
            assert!(!while_node.body.is_empty());
        }
    }

    #[test]
    fn test_parse_plpgsql_error() {
        let result = parse_plpgsql("not valid plpgsql");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_plpgsql_multiple_functions() {
        let result = parse_plpgsql(
            "
            CREATE FUNCTION foo() RETURNS void AS $$ BEGIN NULL; END; $$ LANGUAGE plpgsql;
            CREATE FUNCTION bar() RETURNS int AS $$ BEGIN RETURN 1; END; $$ LANGUAGE plpgsql;
        ",
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        // function() returns None when multiple functions present
        assert!(result.function().is_none());

        // Use functions() for multiple
        assert_eq!(result.functions().len(), 2);
        assert_eq!(result.functions()[0].fn_signature, "foo");
        assert_eq!(result.functions()[1].fn_signature, "bar");
    }
}
