# PL/pgSQL Support

The Postgres Language Server partially supports `PL/pgSQL`. Out of the box, the language server parses the function body with `libpg_query` and reports any syntax error, without needing a database. This shortens the feedback loop while you write the function, but `libpg_query` does not report where in the body the error is, so the error covers the entire function body.

To get syntax errors at the offending token, and many more checks, we strongly recommend to enable the [`plpgsql_check`](https://github.com/okbob/plpgsql_check) extension in your development database.

```sql
CREATE EXTENSION IF NOT EXISTS plpgsql_check;
```

The language server will automatically detect the extension and start forwarding its reports as diagnostics.

For any `CREATE FUNCTION` statement with the language `PL/pgSQL`, the following process occurs:

1. The language server creates the function in a temporary transaction
2. Calls `plpgsql_check_function()` to perform comprehensive static analysis of the function body
3. For trigger functions, the analysis runs against each table that has triggers using this function, providing context-specific validation
4. Errors are mapped back to source locations with token-level precision

When `plpgsql_check` checks a function, it also reports its syntax errors at the offending token (only an error at the very end of the body still covers the whole body), so the language server no longer reports the `libpg_query` syntax error on the entire function body. The `libpg_query` error is still reported when `plpgsql_check` cannot check the function: when there is no database connection, the extension is not installed or the integration is disabled, the function cannot be created in the database, or no trigger uses a trigger function.

The integration provides more detailed and actionable feedback compared to basic syntax checking, including:

> - checks fields of referenced database objects and types inside embedded SQL
> - validates you are using the correct types for function parameters
> - identifies unused variables and function arguments, unmodified OUT arguments
> - partial detection of dead code (code after an RETURN command)
> - detection of missing RETURN command in function (common after exception handlers, complex logic)
> - tries to identify unwanted hidden casts, which can be a performance issue like unused indexes
> - ability to collect relations and functions used by function
> - ability to check EXECUTE statements against SQL injection vulnerability

You can always disable the integration if you do not want the language server to hit your development database.

```json
{
  "plpgsqlCheck": {
    "enabled": false
  }
}
```

