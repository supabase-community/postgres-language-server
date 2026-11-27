use std::ffi::{CStr, CString};
use std::sync::LazyLock;

use prost::Message;
use regex::Regex;

use crate::bindings::*;
use crate::error::*;
use crate::parse::decode_error;
use crate::protobuf::{
    self, PLpgSqlDatum, PLpgSqlExpr, PLpgSqlFunction, PLpgSqlStmt, p_lpg_sql_datum,
    p_lpg_sql_stmt::Node,
};

/// Parses the PL/pgSQL functions of the `CREATE FUNCTION`, `CREATE PROCEDURE` and `DO` statements
/// in `stmt`, one per statement, and returns libpg_query's parse trees of them. Their SQL
/// statements and expressions are kept as text, see [`PLpgSqlExpr`].
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
/// assert_eq!(result.unwrap().functions().len(), 1);
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
            std::slice::from_raw_parts(result.parse_tree.data as *const u8, result.parse_tree.len)
        };
        protobuf::PLpgSqlParseResult::decode(data)
            .map_err(decode_error)
            .map(PlpgsqlParseResult::new)
    };
    unsafe { pg_query_free_plpgsql_protobuf_parse_result(result) };
    parse_result
}

/// The result of parsing PL/pgSQL functions, see [`parse_plpgsql`].
#[derive(Debug)]
pub struct PlpgsqlParseResult {
    /// The parsed protobuf result
    pub protobuf: protobuf::PLpgSqlParseResult,
}

impl PlpgsqlParseResult {
    pub fn new(protobuf: protobuf::PLpgSqlParseResult) -> Self {
        Self { protobuf }
    }

    /// The parsed function, if there is exactly one.
    pub fn function(&self) -> Option<&PLpgSqlFunction> {
        match self.protobuf.functions.as_slice() {
            [function] => Some(function),
            _ => None,
        }
    }

    /// The parsed functions, one per statement of the input that has a PL/pgSQL body.
    pub fn functions(&self) -> &[PLpgSqlFunction] {
        &self.protobuf.functions
    }

    pub fn into_functions(self) -> Vec<PLpgSqlFunction> {
        self.protobuf.functions
    }
}

/// The `parse_mode` of an expression that is a whole SQL statement, [`RAW_PARSE_DEFAULT`]. The
/// others are expressions and assignments, like `x > 0` and `x := f(y)`.
///
/// [`RAW_PARSE_DEFAULT`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/parser/parser.h#L41
const RAW_PARSE_DEFAULT: i32 = 0;

impl PLpgSqlExpr {
    /// Whether the expression is a whole SQL statement, like the `SELECT ...` of
    /// `FOR r IN SELECT ...`, and not an expression like `x > 0`.
    pub fn is_statement(&self) -> bool {
        self.parse_mode == RAW_PARSE_DEFAULT
    }
}

impl PLpgSqlFunction {
    /// The statements of the function's body, depth first in source order, nested ones
    /// included: those of blocks, branches, loops and exception handlers.
    pub fn statements(&self) -> impl Iterator<Item = &PLpgSqlStmt> {
        let mut stack: Vec<&PLpgSqlStmt> = Vec::new();
        if let Some(action) = &self.action {
            let handlers = action.exceptions.iter().flat_map(|block| &block.exc_list);
            stack.extend(handlers.flat_map(|handler| &handler.action).rev());
            stack.extend(action.body.iter().rev());
        }
        std::iter::from_fn(move || {
            let stmt = stack.pop()?;
            let mut children = stmt.children();
            children.reverse();
            stack.extend(children);
            Some(stmt)
        })
    }

    /// The expressions of the function: the defaults and cursor queries of its variables, and
    /// those of [`Self::statements`].
    pub fn expressions(&self) -> impl Iterator<Item = &PLpgSqlExpr> {
        self.datums
            .iter()
            .flat_map(PLpgSqlDatum::expressions)
            .chain(self.statements().flat_map(PLpgSqlStmt::expressions))
    }
}

impl PLpgSqlDatum {
    /// The default value of a variable, and the query of a cursor variable.
    pub fn expressions(&self) -> Vec<&PLpgSqlExpr> {
        match &self.node {
            Some(p_lpg_sql_datum::Node::Var(var)) => [&var.default_val, &var.cursor_explicit_expr]
                .into_iter()
                .flatten()
                .collect(),
            Some(p_lpg_sql_datum::Node::Row(row)) => row.default_val.iter().collect(),
            Some(p_lpg_sql_datum::Node::Rec(rec)) => rec.default_val.iter().collect(),
            Some(p_lpg_sql_datum::Node::Recfield(_)) | None => Vec::new(),
        }
    }
}

impl PLpgSqlStmt {
    /// The statements directly nested in this one, in source order.
    pub fn children(&self) -> Vec<&PLpgSqlStmt> {
        match &self.node {
            Some(Node::StmtBlock(block)) => {
                let handlers = block.exceptions.iter().flat_map(|block| &block.exc_list);
                block
                    .body
                    .iter()
                    .chain(handlers.flat_map(|handler| &handler.action))
                    .collect()
            }
            Some(Node::StmtIf(stmt)) => stmt
                .then_body
                .iter()
                .chain(stmt.elsif_list.iter().flat_map(|elsif| &elsif.stmts))
                .chain(&stmt.else_body)
                .collect(),
            Some(Node::StmtCase(stmt)) => stmt
                .case_when_list
                .iter()
                .flat_map(|when| &when.stmts)
                .chain(&stmt.else_stmts)
                .collect(),
            Some(Node::StmtLoop(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtWhile(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtFori(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtFors(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtForc(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtForeachA(stmt)) => stmt.body.iter().collect(),
            Some(Node::StmtDynfors(stmt)) => stmt.body.iter().collect(),
            _ => Vec::new(),
        }
    }

    /// The expressions of this statement, not those of the statements nested in it.
    pub fn expressions(&self) -> Vec<&PLpgSqlExpr> {
        let Some(node) = &self.node else {
            return Vec::new();
        };
        let (exprs, lists): (Vec<&Option<PLpgSqlExpr>>, Vec<&Vec<PLpgSqlExpr>>) = match node {
            Node::StmtBlock(_)
            | Node::StmtLoop(_)
            | Node::StmtGetdiag(_)
            | Node::StmtClose(_)
            | Node::StmtCommit(_)
            | Node::StmtRollback(_) => (vec![], vec![]),
            Node::StmtAssign(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtIf(stmt) => {
                let mut exprs = vec![&stmt.cond];
                exprs.extend(stmt.elsif_list.iter().map(|elsif| &elsif.cond));
                (exprs, vec![])
            }
            Node::StmtCase(stmt) => {
                let mut exprs = vec![&stmt.t_expr];
                exprs.extend(stmt.case_when_list.iter().map(|when| &when.expr));
                (exprs, vec![])
            }
            Node::StmtWhile(stmt) => (vec![&stmt.cond], vec![]),
            Node::StmtFori(stmt) => (vec![&stmt.lower, &stmt.upper, &stmt.step], vec![]),
            Node::StmtFors(stmt) => (vec![&stmt.query], vec![]),
            Node::StmtForc(stmt) => (vec![&stmt.argquery], vec![]),
            Node::StmtForeachA(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtExit(stmt) => (vec![&stmt.cond], vec![]),
            Node::StmtReturn(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtReturnNext(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtReturnQuery(stmt) => (vec![&stmt.query, &stmt.dynquery], vec![&stmt.params]),
            Node::StmtRaise(stmt) => (
                stmt.options.iter().map(|option| &option.expr).collect(),
                vec![&stmt.params],
            ),
            Node::StmtAssert(stmt) => (vec![&stmt.cond, &stmt.message], vec![]),
            Node::StmtExecsql(stmt) => (vec![&stmt.sqlstmt], vec![]),
            Node::StmtDynexecute(stmt) => (vec![&stmt.query], vec![&stmt.params]),
            Node::StmtDynfors(stmt) => (vec![&stmt.query], vec![&stmt.params]),
            Node::StmtOpen(stmt) => (
                vec![&stmt.argquery, &stmt.query, &stmt.dynquery],
                vec![&stmt.params],
            ),
            Node::StmtFetch(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtPerform(stmt) => (vec![&stmt.expr], vec![]),
            Node::StmtCall(stmt) => (vec![&stmt.expr], vec![]),
        };
        exprs
            .into_iter()
            .flatten()
            .chain(lists.into_iter().flatten())
            .collect()
    }

    /// The expression whose value is the SQL statement that this statement runs, if it runs
    /// dynamic SQL: `EXECUTE`, `FOR ... IN EXECUTE`, `RETURN QUERY EXECUTE` and
    /// `OPEN ... FOR EXECUTE`.
    pub fn dynamic_query(&self) -> Option<&PLpgSqlExpr> {
        match self.node.as_ref()? {
            Node::StmtDynexecute(stmt) => stmt.query.as_ref(),
            Node::StmtDynfors(stmt) => stmt.query.as_ref(),
            Node::StmtReturnQuery(stmt) => stmt.dynquery.as_ref(),
            Node::StmtOpen(stmt) => stmt.dynquery.as_ref(),
            _ => None,
        }
    }
}

/// Messages libpg_query reports on valid PL/pgSQL functions because it compiles them without a
/// catalog: it only knows the types of `pg_catalog`, reads an unknown type in `public` as
/// `record`, so an array of one as `_record`, finds no element type for a `VARIADIC` array, and
/// fails with `Not implemented` on anything else, e.g. a type in another schema.
/// https://github.com/pganalyze/libpg_query/blob/18.1.0/src/postgres/src_backend_utils_cache_syscache.c#L332-L344
const NO_CATALOG_ERRORS: &[&str] = &[
    "Not implemented",
    "cannot accept type _record",
    "cannot return type _record",
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
    use crate::protobuf::{PLpgSqlStmtBlock, p_lpg_sql_datum::Node as Datum};

    /// The body of the one function of `sql`.
    fn body(sql: &str) -> PLpgSqlStmtBlock {
        let function = parse_plpgsql(sql).unwrap().into_functions().remove(0);
        function.action.unwrap()
    }

    #[test]
    fn parses_a_simple_function() {
        let result = parse_plpgsql(
            "CREATE OR REPLACE FUNCTION test_func() RETURNS void AS $$
            BEGIN
                NULL;
            END;
            $$ LANGUAGE plpgsql;",
        )
        .unwrap();
        let function = result.function().unwrap();
        // `NULL;` compiles to nothing, PL/pgSQL adds a `RETURN` at the end of the body.
        let action = function.action.as_ref().unwrap();
        assert!(matches!(
            action.body.as_slice(),
            [PLpgSqlStmt {
                node: Some(Node::StmtReturn(_))
            }]
        ));
    }

    #[test]
    fn parses_assignments() {
        let body = body(
            "CREATE FUNCTION add_numbers(a int, b int) RETURNS int AS $$
            DECLARE
                result int;
            BEGIN
                result := a + b;
                RETURN result;
            END;
            $$ LANGUAGE plpgsql;",
        );
        let [assign, ret, ..] = body.body.as_slice() else {
            panic!("{body:#?}");
        };
        let Some(Node::StmtAssign(assign)) = &assign.node else {
            panic!("{assign:#?}");
        };
        // The expression of an assignment is the whole assignment.
        assert_eq!(assign.expr.as_ref().unwrap().query, "result := a + b");
        assert!(matches!(ret.node, Some(Node::StmtReturn(_))));
    }

    #[test]
    fn parses_if_conditions() {
        let body = body(
            "CREATE FUNCTION cs_fmt_browser_version(v_name varchar, v_version varchar)
            RETURNS varchar AS $$
            BEGIN
                IF v_version IS NULL THEN
                    RETURN v_name;
                END IF;
                RETURN v_name || '/' || v_version;
            END;
            $$ LANGUAGE plpgsql;",
        );
        let Some(Node::StmtIf(stmt)) = &body.body[0].node else {
            panic!("{body:#?}");
        };
        assert_eq!(stmt.cond.as_ref().unwrap().query, "v_version IS NULL");
        assert!(matches!(stmt.then_body[0].node, Some(Node::StmtReturn(_))));
        assert!(matches!(body.body[1].node, Some(Node::StmtReturn(_))));
    }

    #[test]
    fn parses_while_loops() {
        let body = body(
            "CREATE FUNCTION count_down(n int) RETURNS void AS $$
            BEGIN
                WHILE n > 0 LOOP
                    n := n - 1;
                END LOOP;
            END;
            $$ LANGUAGE plpgsql;",
        );
        let Some(Node::StmtWhile(stmt)) = &body.body[0].node else {
            panic!("{body:#?}");
        };
        assert_eq!(stmt.cond.as_ref().unwrap().query, "n > 0");
        assert!(matches!(stmt.body[0].node, Some(Node::StmtAssign(_))));
    }

    #[test]
    fn reports_syntax_errors() {
        let err = parse_plpgsql(
            "create function f() returns int language plpgsql as $$ begin retur 1; end $$",
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::Parse(r#"syntax error at or near "retur""#.to_string())
        );
    }

    #[test]
    fn parses_one_function_per_statement() {
        let result = parse_plpgsql(
            "CREATE FUNCTION foo() RETURNS void AS $$ BEGIN NULL; END; $$ LANGUAGE plpgsql;
            DO $$ BEGIN PERFORM 1; END $$;
            CREATE FUNCTION bar() RETURNS int AS 'select 1' LANGUAGE sql;",
        )
        .unwrap();
        assert!(result.function().is_none());
        let functions = result.functions();
        assert_eq!(functions.len(), 3);
        // libpg_query has no catalog to format the signature from.
        assert_eq!(functions[0].fn_signature, "plpgsql_function");
        assert_eq!(functions[1].fn_signature, "inline_code_block");
        // A function in another language has no body.
        assert!(functions[2].action.is_none());
    }

    #[test]
    fn walks_nested_blocks_and_exception_handlers() {
        let result = parse_plpgsql(
            "create function f(a int) returns void language plpgsql as $$
            begin
              begin
                if a > 0 then
                  insert into t values (a);
                elsif a < 0 then
                  delete from t;
                else
                  perform g();
                end if;
              exception
                when division_by_zero or unique_violation then
                  raise notice 'failed: %', sqlerrm;
                when others then
                  null;
              end;
            exception when others then
              update t set a = 0;
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        // The handlers of the function's block are those of a block that PL/pgSQL wraps the
        // body in.
        let conditions: Vec<Vec<_>> = function
            .statements()
            .filter_map(|stmt| match &stmt.node {
                Some(Node::StmtBlock(block)) => block.exceptions.as_ref(),
                _ => None,
            })
            .flat_map(|exceptions| &exceptions.exc_list)
            .map(|handler| {
                handler
                    .conditions
                    .iter()
                    .map(|condition| condition.condname.as_str())
                    .collect()
            })
            .collect();
        assert_eq!(
            conditions,
            [
                vec!["others"],
                vec!["division_by_zero", "unique_violation"],
                vec!["others"],
            ]
        );
        let expressions: Vec<_> = function
            .expressions()
            .map(|expr| expr.query.as_str())
            .collect();
        assert_eq!(
            expressions,
            [
                "a > 0",
                "a < 0",
                "insert into t values (a)",
                "delete from t",
                "SELECT g()",
                "sqlerrm",
                "update t set a = 0",
            ]
        );
    }

    #[test]
    fn parses_for_loops_over_queries() {
        let result = parse_plpgsql(
            "create function f() returns setof int language plpgsql as $$
            declare r record;
            begin
              for r in select id from t where id > 0 loop
                return next r.id;
              end loop;
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        let Some(Node::StmtFors(stmt)) = &function.action.as_ref().unwrap().body[0].node else {
            panic!("{function:#?}");
        };
        let query = stmt.query.as_ref().unwrap();
        assert_eq!(query.query, "select id from t where id > 0");
        assert!(query.is_statement());
        // The loop variable is an index into the datums.
        let Some(Datum::Rec(rec)) = &function.datums[stmt.var_dno as usize].node else {
            panic!("{function:#?}");
        };
        assert_eq!(rec.refname, "r");
        let Some(Node::StmtReturnNext(next)) = &stmt.body[0].node else {
            panic!("{stmt:#?}");
        };
        let expr = next.expr.as_ref().unwrap();
        assert_eq!(expr.query, "r.id");
        assert!(!expr.is_statement());
    }

    #[test]
    fn parses_return_query() {
        let result = parse_plpgsql(
            "create function f(a int) returns setof int language plpgsql as $$
            begin
              return query select a;
              return query execute 'select $1' using a;
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        let body = &function.action.as_ref().unwrap().body;
        let Some(Node::StmtReturnQuery(stmt)) = &body[0].node else {
            panic!("{function:#?}");
        };
        assert_eq!(stmt.query.as_ref().unwrap().query, "select a");
        assert!(body[0].dynamic_query().is_none());
        let dynamic = body[1].dynamic_query().unwrap();
        assert_eq!(dynamic.query, "'select $1'");
        assert_eq!(body[1].expressions().len(), 2);
    }

    #[test]
    fn parses_dynamic_execute() {
        let result = parse_plpgsql(
            "create function f(name text) returns void language plpgsql as $$
            declare n int;
            begin
              execute format('select count(*) from %I', name) into strict n using name;
              for n in execute 'select 1' loop end loop;
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        let body = &function.action.as_ref().unwrap().body;
        let Some(Node::StmtDynexecute(stmt)) = &body[0].node else {
            panic!("{function:#?}");
        };
        assert!(stmt.into && stmt.strict);
        assert_eq!(
            body[0].dynamic_query().unwrap().query,
            "format('select count(*) from %I', name)"
        );
        assert_eq!(stmt.params[0].query, "name");
        let Some(Datum::Row(row)) = &function.datums[stmt.target_dno as usize].node else {
            panic!("{function:#?}");
        };
        let Some(Datum::Var(n)) = &function.datums[row.fields[0].varno as usize].node else {
            panic!("{function:#?}");
        };
        assert_eq!(n.refname, "n");
        assert_eq!(body[1].dynamic_query().unwrap().query, "'select 1'");
    }

    #[test]
    fn has_the_lines_of_statements() {
        let result = parse_plpgsql(
            "create function f(a int) returns int language plpgsql as $$
            declare
              x int := a;
            begin
              x := x + 1;
              if x > 1 then
                return x;
              end if;
              return 0;
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        let action = function.action.as_ref().unwrap();
        // Lines are counted from the line of the opening `$$`.
        assert_eq!(action.lineno, 4);
        let lines: Vec<_> = function
            .statements()
            .map(|stmt| match &stmt.node {
                Some(Node::StmtAssign(stmt)) => stmt.lineno,
                Some(Node::StmtIf(stmt)) => stmt.lineno,
                Some(Node::StmtReturn(stmt)) => stmt.lineno,
                node => panic!("{node:#?}"),
            })
            .collect();
        // There are no columns or offsets.
        assert_eq!(lines, [5, 6, 7, 9]);
        let Some(Datum::Var(x)) = &function.datums[2].node else {
            panic!("{function:#?}");
        };
        assert_eq!((x.refname.as_str(), x.lineno), ("x", 3));
    }

    #[test]
    fn has_the_types_of_variables() {
        let result = parse_plpgsql(
            "create function f(a int) returns void language plpgsql as $$
            declare
              t text[] not null := '{}';
              v varchar(10);
              c cursor (k int) for select k;
            begin
            end $$",
        )
        .unwrap();
        let function = result.function().unwrap();
        let types: Vec<_> = function
            .datums
            .iter()
            .filter_map(|datum| match &datum.node {
                Some(Datum::Var(var)) => {
                    let datatype = var.datatype.as_ref().unwrap();
                    Some((
                        var.refname.as_str(),
                        datatype.typname.as_str(),
                        datatype.typoid,
                        datatype.atttypmod,
                        datatype.origtypname.is_some(),
                    ))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            types,
            [
                ("a", "int4", 23, -1, false),
                ("found", "bool", 16, -1, false),
                ("t", "_text", 1009, -1, false),
                // Without a catalog, the type modifier is lost, and the type name as written is
                // only kept for composite types.
                ("v", "varchar", 1043, -1, false),
                ("k", "int4", 23, -1, false),
                ("c", "refcursor", 1790, -1, false),
            ]
        );
        let defaults: Vec<_> = function
            .expressions()
            .map(|expr| expr.query.as_str())
            .collect();
        assert_eq!(defaults, ["'{}'", "select k"]);
    }

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
        ] {
            let err = parse_plpgsql(input).expect_err(input);
            assert!(err.is_false_positive_without_catalog(), "{input}: {err}");
        }
    }
}
