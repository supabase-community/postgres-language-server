//! What running a PL/pgSQL function may do, from libpg_query's parse tree of its body.

use pgls_query::{
    Identifier, NodeEnum,
    protobuf::{PLpgSqlExpr, Token, a_const},
};

use crate::is_plain_query;

/// How many statements and expressions of a function's body [`may_change_catalog`] looks at. A
/// hundred thousand only stops generated input, and looking at them takes a few milliseconds.
const PLPGSQL_NODES_MAX: usize = 100_000;

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
/// It may also when the body doesn't parse, or has more than [`PLPGSQL_NODES_MAX`] statements
/// and expressions.
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
    let Ok(parsed) = pgls_query::parse_plpgsql(create_function) else {
        return true;
    };
    let Some(function) = parsed.function() else {
        return true;
    };
    let mut visited = 0;
    let mut too_many = || {
        visited += 1;
        let too_many = visited > PLPGSQL_NODES_MAX;
        if too_many {
            tracing::warn!(
                "stopped looking at a PL/pgSQL function body after {PLPGSQL_NODES_MAX} statements and expressions"
            );
        }
        too_many
    };
    for stmt in function.statements() {
        if too_many() {
            return true;
        }
        if let Some(dynamic) = stmt.dynamic_query() {
            let Some(sql) = string_constant(&dynamic.query) else {
                return true;
            };
            if statement_may_change_catalog(&sql, &calls_catalog_function) {
                return true;
            }
        }
    }
    for expr in function.expressions() {
        if too_many() {
            return true;
        }
        if expression_may_change_catalog(
            expr,
            &calls_catalog_function,
            &is_catalog_function,
            any_catalog_function,
        ) {
            return true;
        }
    }
    false
}

/// Whether running `expr`, an expression of the body, may change the catalog, see
/// [`may_change_catalog`].
fn expression_may_change_catalog(
    expr: &PLpgSqlExpr,
    calls_catalog_function: &impl Fn(&NodeEnum) -> bool,
    is_catalog_function: &impl Fn(&str) -> bool,
    any_catalog_function: bool,
) -> bool {
    if expr.is_statement() {
        return statement_may_change_catalog(&expr.query, calls_catalog_function);
    }
    any_catalog_function && calls_by_name(&expr.query, is_catalog_function)
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
