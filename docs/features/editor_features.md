# Editor Features

When connected to a database, the language server helps you write SQL with autocompletion, hover information, inlay hints, and semantic highlighting.

Hover, inlay hints, and semantic highlighting follow Postgres' name resolution. They see the database schema with the DDL of the current file applied on top of it, so a table created or altered earlier in the file is known, and the search path follows `set search_path` in the file. They also work in the bodies of `LANGUAGE sql` functions, where the function's parameters are in scope.

## Requirements

All these features need:

- A configured database connection. See the [database configuration guide](../guides/configure_database.md).
- A database user that can read the schema.

Without a database connection, they are not available. The language server caches the schema when it connects. Run the `pgls.invalidateSchemaCache` command after changing the schema outside the editor.

## Autocompletion

As you type SQL, the language server suggests relevant database objects based on your current context:

- **Tables**: Available tables from your database schema
- **Columns**: Columns from tables referenced in your query
- **Functions**: Database functions and procedures
- **Schemas**: Available schemas in your database
- **Keywords**: SQL keywords and syntax

The suggestions are context-aware - for example, when typing after `FROM`, you'll see table suggestions, and when typing after `SELECT`, you'll see column suggestions from relevant tables.

## Hover

Hovering a name shows the object it refers to:

- **Tables and views**: schema, columns with their types, and defaults.
- **Columns**: type, nullability, and the table they belong to. Columns are followed through aliases and joins, so `o.name` shows the column of the table that `o` stands for.
- **Columns of CTEs and subqueries**: their name and inferred type.
- **Functions**: only the overload that the call resolves to, with its signature and return type.
- **Schemas, types, and roles.**

```sql
create function pick(i integer) returns integer language sql as 'select i';
create function pick(t text) returns text language sql as 'select t';
select pick(1);   -- hover on `pick`: only `public.pick(i integer) → integer`
```

When a name doesn't resolve to exactly one object, hover falls back to looking the name up in the schema and shows every match.

## Inlay Hints

Function calls show the names of the parameters their positional arguments are passed to:

```sql
select make_point(1, 2 + 3);      -- make_point(x: 1, y: 2 + 3)
select make_point(1, y => 2);     -- make_point(x: 1, y => 2)
select sum_all(1, 2, 3);          -- sum_all(base: 1, nums: 2, 3)
```

Hints are only shown when the call resolves to exactly one function, including functions created earlier in the same file. Named arguments (`f(a => 1)`), unnamed parameters, and arguments that already carry the parameter's name (`f(t.x)` for the parameter `x`) get no hint. Of the elements passed to a `VARIADIC` parameter, only the first is labelled.

## Semantic Highlighting

The language server provides semantic tokens for the names that resolve to exactly one database object, so your editor can color them by what they refer to rather than by how they look:

| Token | LSP token type | Example |
| --- | --- | --- |
| Schema | `namespace` | `public` in `public.users` |
| Table, view, or other relation | `class` | `users` in `from users`, `users.id` |
| Column | `property` | `name`, `u.name`, `public.users.id` |
| Function | `function` | the overload that a call resolves to |
| Type | `type` | `int8` in `x::int8`, `cast(x as text)`, `date 'today'` |
| Parameter of a SQL function, in its body | `parameter` | `id`, `fn.id`, `$1` |
| Enum value | `enumMember` | `'active'` in `status = 'active'` |

Objects in `pg_catalog` and `information_schema` carry the `defaultLibrary` modifier. Names that match no object or several get no token and keep the highlighting of your editor's grammar.

String literals are enum values only where the enum type is certain: casts (`'active'::status`), comparisons with an enum column or value (`=`, `<>`, `IN`, `BETWEEN`, `IS DISTINCT FROM`, `NULLIF`), arguments of function parameters with the enum type, and values assigned to enum columns in `INSERT ... VALUES`, `UPDATE ... SET`, and `ON CONFLICT DO UPDATE SET`.

## Turning Features Off

Inlay hints and semantic highlighting are on by default. Turn them off for a project in `postgres-language-server.jsonc`:

```jsonc
{
  "inlayHints": { "enabled": false },
  "semanticTokens": { "enabled": false }
}
```

Without semantic tokens, your editor keeps the highlighting of its SQL grammar. Most editors can also turn these features off on their side, e.g. `editor.inlayHints.enabled` and `editor.semanticHighlighting.enabled` in VS Code.

The language server asks the editor to request inlay hints and semantic tokens again when the configuration changes, when the schema cache is invalidated or a statement is executed, and when the database becomes reachable, if the editor supports these refreshes.

## Limitations

- Only names that resolve to exactly one object get a hover from the resolver, an inlay hint, or a token. Ambiguous calls, unknown objects, and constructs the resolver doesn't model are skipped.
- After a `DO` block, `CALL`, `EXECUTE`, or a statement that doesn't parse, the rest of the file gets no function hints, function tokens, or enum tokens, because their effects on the schema are unknown. Relations and columns still resolve.
- `CALL proc(...)` gets no hints, tokens, or resolved hover.
- Statements in `BEGIN ATOMIC` function bodies and in `'...'`-quoted bodies that contain `''` are not resolved. `$$`-quoted bodies are. PL/pgSQL bodies are not covered.
- No tokens for roles, for schemas outside qualified names, for DDL (column types in `create table`, function signatures), for CTE names, or for table aliases. Tokens that span several lines are skipped.
- Each request resolves the file up to the visible range. On very long files, like schema dumps with thousands of statements, hints and tokens take longer to appear.
