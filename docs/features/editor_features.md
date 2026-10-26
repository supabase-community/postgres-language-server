# Autocompletion & Hover

The language server provides autocompletion and hover information when connected to a database.

## Autocompletion

As you type SQL, the language server suggests relevant database objects based on your current context:

- **Tables**: Available tables from your database schema  
- **Columns**: Columns from tables referenced in your query  
- **Functions**: Database functions and procedures  
- **Schemas**: Available schemas in your database  
- **Keywords**: SQL keywords and syntax  

The suggestions are context-aware - for example, when typing after `FROM`, you'll see table suggestions, and when typing after `SELECT`, you'll see column suggestions from relevant tables.

## Hover Information

Hovering over database objects in your SQL shows detailed information:

- **Tables**: Schema, column list with data types  
- **Columns**: Data type, nullable status, table location  
- **Functions**: Return type, parameter information  

The hover information is pulled from your database schema.

## Requirements

Both features require:  
- A configured database connection  
- The language server must be able to read schema information from your database  

Without a database connection, these features are not available.

## Configuration

These features work automatically when you have a database connection configured. See the [database configuration guide](../guides/configure_database.md) for setup instructions.

The language server caches schema information on startup.

## Inlay Hints

Function calls show the names of the parameters their positional arguments are passed to, as in `make_point(x: 1, y: 2)`. Hints are only shown when the call resolves to exactly one function, including functions created earlier in the same file. Named arguments (`f(a => 1)`), unnamed parameters, and arguments that already carry the parameter's name (`f(t.x)` for the parameter `x`) get no hint. Of the elements passed to a `VARIADIC` parameter, only the first is labelled.

Inlay hints require a database connection. Enable them in your editor, e.g. with `editor.inlayHints.enabled` in VS Code.

## Semantic Highlighting

The language server provides semantic tokens for the names that resolve to exactly one database object, so your editor can color them by what they refer to rather than by how they look:

- **Schemas** (`namespace`), like `public` in `public.users`
- **Tables, views, and other relations** (`class`)
- **Columns** (`property`)
- **Functions** (`function`), the overload that the call resolves to
- **Types** (`type`), like `int8` in `x::int8`
- **Parameters** (`parameter`) of a SQL function, in its body, by name or as `$1`
- **Enum values** (`enumMember`): string literals that are coerced to an enum type, like `'active'` in `status = 'active'`, `'active'::status`, or `insert into t (status) values ('active')`

Objects in `pg_catalog` and `information_schema` carry the `defaultLibrary` modifier. Names follow Postgres' name resolution, including objects created earlier in the same file and statements in the bodies of SQL functions. Names that match no object or several get no token and keep the highlighting of your editor's grammar.

Semantic highlighting requires a database connection. In VS Code, it is controlled by `editor.semanticHighlighting.enabled`.

## Expand `*`

With the cursor on a `*` or `t.*` in a select list or `RETURNING` list, the refactor code action "Expand `*` into its columns" replaces it with the columns it stands for:

```sql
select * from users u join posts p on p.user_id = u.id;
-- becomes
select u.id, u.name, p.id, p.user_id, p.title from users u join posts p on p.user_id = u.id;
```

Columns are qualified with the name or alias of their table when there are several in `FROM`, and quoted where needed. The columns of CTEs, subqueries, and tables created earlier in the file are expanded too. The action is only offered when every column is known and can be referenced unambiguously, so not for joins with `USING` or `NATURAL`, tables whose columns are unknown, or sources with two columns of the same name.

Like hover, it requires a database connection.
