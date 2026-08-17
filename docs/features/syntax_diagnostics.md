# Syntax Diagnostics

The Postgres Language Server reports diagnostics for syntax errors in your SQL files. Syntax diagnostics are enabled by default and cannot be disabled.

## How it Works

The language server first splits SQL files into individual statements. It then uses [libpg_query](https://github.com/pganalyze/libpg_query), which packages the actual Postgres parser, to validate each statement against Postgres syntax.

When you type or modify SQL, the language server:

1. Splits the file into individual statements
2. Parses each statement using `libpg_query`
3. Reports any syntax errors as diagnostics

### Statement Boundaries

The statement splitter recognizes semicolons and blank lines as statement boundaries. This allows the language server to analyze statements while they are being written, before they have a terminating semicolon.

Avoid blank lines where a single Postgres statement must continue, such as between the final common table expression (CTE) and its main query. For example, write:

```sql
WITH foo AS (
    SELECT 1 AS id
)
SELECT id
FROM foo;
```

Use blank lines between complete statements instead.

## Features

- Postgres-compatible parsing: Uses the same parser as Postgres itself for accurate syntax validation
- Named Parameter Support: We convert `:param` and `@param` to positional parameters (`$1`, `$2`) so the Postgres parser understands them and the LSP works with ORMs and other tooling  
- `PL/pgSQL`: In addition to SQL, also validates `PL/pgSQL` function bodies for basic syntax errors  

## Error Information

Syntax errors include:  
- The exact error message from the Postgres parser  
- Source location when available (though `libpg_query` does not always provide precise positions)  
- Error severity (always "Error" for syntax issues)  

Note: For more advanced `PL/pgSQL` validation beyond basic syntax, see the [PL/pgSQL feature](plpgsql.md) which integrates with the `plpgsql_check` extension.
