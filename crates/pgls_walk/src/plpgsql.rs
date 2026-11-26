//! What running a PL/pgSQL function may do, from libpg_query's parse tree of its body.

use pgls_query::{
    Identifier, NodeEnum,
    protobuf::{Token, a_const},
};
use serde_json::Value;

use crate::is_plain_query;

/// How many JSON values of a function's parse tree [`may_change_catalog`] visits. A statement of
/// the body takes about ten; a hundred thousand only stops generated input, and visiting them
/// takes a few milliseconds.
const PLPGSQL_VALUES_MAX: usize = 100_000;

/// The `parseMode` of an expression that is a whole SQL statement, [`RAW_PARSE_DEFAULT`]. The
/// others are expressions and assignments, like `x > 0` and `x := f(y)`.
///
/// [`RAW_PARSE_DEFAULT`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/parser/parser.h#L41
const RAW_PARSE_DEFAULT: i64 = 0;

/// Whether running the PL/pgSQL function that `create_function`, the text of its
/// `CREATE FUNCTION` statement, creates may change the catalog. It may if its body has
///
/// - a SQL statement that isn't a plain query, see [`is_plain_query`]. PL/pgSQL takes the
///   `INTO` of a `SELECT` out of the statement, so `SELECT ... INTO var` is a plain query, and
///   `EXPLAIN` runs at most the statement it explains, like the `EXPLAIN` wrappers of Postgres'
///   regression tests: `FOR line IN EXPLAIN (ANALYZE) SELECT ...`.
/// - dynamic SQL: `EXECUTE`, `FOR ... IN EXECUTE`, `OPEN ... FOR EXECUTE` and
///   `RETURN QUERY EXECUTE`, whose statements are only known when they run. A string constant,
///   like `EXECUTE 'DROP INDEX i'`, is the statement it runs.
/// - a call of a function for which `calls_catalog_function` is true. The statements of the
///   body are parsed and passed to it; in expressions, like `x := f(y)`, any name followed by
///   `(` counts as a call to the function `is_catalog_function` tells about.
///
/// It may also when the body doesn't parse, or has more than [`PLPGSQL_VALUES_MAX`] values.
///
/// Cost: parsing the body with libpg_query's PL/pgSQL parser, and each of its SQL statements
/// with the SQL parser. Expressions are only scanned when the file created a function that may
/// change the catalog.
pub(crate) fn may_change_catalog(
    create_function: &str,
    calls_catalog_function: impl Fn(&NodeEnum) -> bool,
    is_catalog_function: impl Fn(&str) -> bool,
    any_catalog_function: bool,
) -> bool {
    let Ok(json) = pgls_query::parse_plpgsql_json(create_function) else {
        return true;
    };
    // serde_json stops at a nesting depth of 128 and returns an error.
    let Ok(tree) = serde_json::from_str::<Value>(&json) else {
        return true;
    };
    let mut stack = vec![&tree];
    let mut visited = 0;
    while let Some(value) = stack.pop() {
        visited += 1;
        if visited > PLPGSQL_VALUES_MAX {
            tracing::warn!(
                "stopped looking at a PL/pgSQL function body after {PLPGSQL_VALUES_MAX} values"
            );
            return true;
        }
        match value {
            Value::Array(values) => stack.extend(values),
            Value::Object(fields) => {
                for (key, value) in fields {
                    let dynamic = match key.as_str() {
                        "PLpgSQL_stmt_dynexecute" | "PLpgSQL_stmt_dynfors" => value.get("query"),
                        "dynquery" => Some(value),
                        _ => None,
                    };
                    if let Some(dynamic) = dynamic {
                        let constant = dynamic
                            .get("PLpgSQL_expr")
                            .and_then(|expr| expr.get("query"))
                            .and_then(Value::as_str)
                            .and_then(string_constant);
                        let Some(sql) = constant else {
                            return true;
                        };
                        if statement_may_change_catalog(&sql, &calls_catalog_function) {
                            return true;
                        }
                    }
                    match key.as_str() {
                        "PLpgSQL_expr" => {
                            if expression_may_change_catalog(
                                value,
                                &calls_catalog_function,
                                &is_catalog_function,
                                any_catalog_function,
                            ) {
                                return true;
                            }
                        }
                        _ => stack.push(value),
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether running `expr`, a `PLpgSQL_expr` of the parse tree, may change the catalog, see
/// [`may_change_catalog`].
fn expression_may_change_catalog(
    expr: &Value,
    calls_catalog_function: &impl Fn(&NodeEnum) -> bool,
    is_catalog_function: &impl Fn(&str) -> bool,
    any_catalog_function: bool,
) -> bool {
    let Some(query) = expr.get("query").and_then(Value::as_str) else {
        return false;
    };
    if expr.get("parseMode").and_then(Value::as_i64) == Some(RAW_PARSE_DEFAULT) {
        return statement_may_change_catalog(query, calls_catalog_function);
    }
    any_catalog_function && calls_by_name(query, is_catalog_function)
}

/// Whether running the SQL statement `sql` may change the catalog: if it isn't a plain query or
/// the `EXPLAIN` of one, calls a function for which `calls_catalog_function` is true, or doesn't
/// parse.
fn statement_may_change_catalog(
    sql: &str,
    calls_catalog_function: &impl Fn(&NodeEnum) -> bool,
) -> bool {
    let Some(root) = pgls_query::parse(sql)
        .ok()
        .and_then(|parsed| parsed.into_root())
    else {
        return true;
    };
    let runs = match &root {
        NodeEnum::ExplainStmt(explain) => {
            match explain
                .query
                .as_deref()
                .and_then(|query| query.node.as_ref())
            {
                Some(query) => query,
                None => return true,
            }
        }
        root => root,
    };
    !is_plain_query(runs) || calls_catalog_function(&root)
}

/// The value of `expr`, an expression of the body, if it is a string constant.
fn string_constant(expr: &str) -> Option<String> {
    let NodeEnum::SelectStmt(select) = pgls_query::parse(&format!("SELECT {expr}"))
        .ok()?
        .into_root()?
    else {
        return None;
    };
    let [target] = select.target_list.as_slice() else {
        return None;
    };
    let Some(NodeEnum::ResTarget(target)) = &target.node else {
        return None;
    };
    let Some(NodeEnum::AConst(constant)) = target.val.as_deref()?.node.as_ref() else {
        return None;
    };
    match &constant.val {
        Some(a_const::Val::Sval(value)) if select.from_clause.is_empty() => {
            Some(value.sval.clone())
        }
        _ => None,
    }
}

/// Whether a name in `expr` that is followed by `(` is one for which `is_function` is true.
/// Expressions that can't be scanned may call any function.
fn calls_by_name(expr: &str, is_function: &impl Fn(&str) -> bool) -> bool {
    let Ok(scanned) = pgls_query::scan(expr) else {
        return true;
    };
    scanned.tokens.windows(2).any(|pair| {
        let [name, next] = pair else {
            return false;
        };
        next.token == Token::Ascii40 as i32
            && usize::try_from(name.start)
                .ok()
                .zip(usize::try_from(name.end).ok())
                .and_then(|(start, end)| expr.get(start..end))
                .is_some_and(|name| is_function(&Identifier::new(name).name()))
    })
}

#[cfg(test)]
mod tests {
    use super::may_change_catalog;

    fn plpgsql(body: &str) -> String {
        format!(
            "create function f(a int) returns void language plpgsql as $$
            declare x int;
            begin
            {body}
            end $$;"
        )
    }

    /// Whether the function with `body` may change the catalog, when `make_table` is the only
    /// function of the file that may.
    fn changes(body: &str) -> bool {
        let is_catalog_function = |name: &str| name == "make_table";
        may_change_catalog(
            &plpgsql(body),
            |stmt| {
                stmt.iter().any(|node| {
                    let pgls_query::NodeRef::FuncCall(call) = node else {
                        return false;
                    };
                    call.funcname
                        .last()
                        .and_then(pgls_query_ext::utils::string_value)
                        .is_some_and(is_catalog_function)
                })
            },
            is_catalog_function,
            true,
        )
    }

    #[test]
    fn statements_that_change_the_catalog() {
        assert!(changes("create table t (a int);"));
        assert!(changes(
            "if a > 0 then create materialized view v as select 1; end if;"
        ));
        assert!(changes("drop index i;"));
        assert!(changes("call p();"));
        assert!(changes(
            "for x in explain analyze create table t as select 1 loop end loop;"
        ));
        assert!(changes(
            "for x in explain select make_table() loop end loop;"
        ));
    }

    #[test]
    fn plain_queries() {
        assert!(!changes("insert into t values (a);"));
        assert!(!changes("select a into x from t;"));
        assert!(!changes("perform other(a);"));
        assert!(!changes(
            "for x in select a from t loop update t set a = x; end loop;"
        ));
        assert!(!changes(
            "for x in explain (analyze, costs off) select a from t loop end loop;"
        ));
        assert!(!changes("x := other(a) + 1; return;"));
    }

    #[test]
    fn dynamic_sql() {
        assert!(changes("execute 'drop index i';"));
        assert!(changes("execute 'select make_table()';"));
        assert!(changes("execute format('select %s', a);"));
        assert!(changes("execute 'select ' || a;"));
        assert!(changes(
            "for x in execute 'create table t ()' loop end loop;"
        ));
        assert!(changes("execute 'explain analyze ' || a;"));
        assert!(!changes("execute 'select 1';"));
        assert!(!changes("for x in execute 'select 1' loop end loop;"));
        assert!(!changes(
            "execute 'select a from t where a = $1' into x using a;"
        ));
    }

    #[test]
    fn calls_to_functions_that_change_the_catalog() {
        assert!(changes("perform make_table();"));
        assert!(changes("insert into t select make_table();"));
        assert!(changes("x := make_table() + 1;"));
        assert!(changes("if \"make_table\"() then return; end if;"));
        assert!(changes("if public.MAKE_TABLE() then return; end if;"));
        assert!(!changes("x := make_table + 1;"));
    }

    #[test]
    fn bodies_that_dont_parse() {
        // libpg_query leaves SQL statements to the SQL parser.
        assert!(changes("create tabel t (a int);"));
        assert!(changes("if a > 0 then return;"));
        assert!(may_change_catalog(
            "create function f() returns void language plpgsql as $$begin$$;",
            |_| false,
            |_| false,
            false,
        ));
    }
}
