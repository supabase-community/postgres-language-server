# Syntax Diagnostics

The Postgres Language Server reports diagnostics for syntax errors in your SQL files. Syntax diagnostics are enabled by default and cannot be disabled.

## How it Works

The language server uses [libpg_query](https://github.com/pganalyze/libpg_query) to parse SQL statements, which is the actual Postgres parser packaged as a library. This ensures 100% compatibility with Postgres syntax.

When you type or modify SQL, the language server:  
1. Parses the SQL using `libpg_query`  
2. Reports any syntax errors as diagnostics

## Features

- Always correct: Uses the same parser as Postgres itself for accurate syntax validation  
- Named Parameter Support: We convert `:param` and `@param` to positional parameters (`$1`, `$2`) so the Postgres parser understands them and the LSP works with ORMs and other tooling  
- `PL/pgSQL`: In addition to SQL, also validates `PL/pgSQL` function bodies for basic syntax errors  

## Error Information

Syntax errors include:  
- The exact error message from the Postgres parser  
- Source location when available (though `libpg_query` does not always provide precise positions)  
- Error severity (always "Error" for syntax issues)  

Note: For more advanced `PL/pgSQL` validation beyond basic syntax, see the [PL/pgSQL feature](plpgsql.md) which integrates with the `plpgsql_check` extension.
