# Type Checking

The Postgres Language Server checks your SQL against your database schema without running it. As you type, it reports what Postgres would reject: tables and columns that don't exist, ambiguous column references, operators and functions that don't match their argument types, invalid casts, and values that can't be assigned to a column.

## How it Works

When the language server connects to your database, it loads a snapshot of its schema: tables, columns, types, functions, operators, and casts. It then applies the statements of the current file on top of that snapshot, so a table created earlier in a migration is known to the statements after it. Nothing is executed against the database.

Each statement is checked against this catalog by the rules of the `typecheck` group. The type checks follow Postgres' own algorithms for coercion, overload resolution, and common types, so they agree with what Postgres does at runtime.

The rules only report what Postgres would certainly reject. Whenever the language server can't be sure, for example after a `DO` block or a `CREATE EXTENSION` whose effects it can't see, it stays silent.

Statements that only reference objects that exist unchanged in the database are also checked with `EXPLAIN` as a fallback. This catches errors the static rules don't model yet, for `SELECT`, `INSERT`, `UPDATE`, and `DELETE` statements.

## What Gets Checked

| Rule | Reports |
| --- | --- |
| [`unknownRelation`](../reference/rules/unknown-relation.md) | `SELECT * FROM user` when the table is named `users` |
| [`unknownColumn`](../reference/rules/unknown-column.md) | `SELECT user_naem FROM users` |
| [`unknownSchema`](../reference/rules/unknown-schema.md), [`unknownFunction`](../reference/rules/unknown-function.md), [`unknownType`](../reference/rules/unknown-type.md) | references to schemas, functions, and types that don't exist |
| [`ambiguousColumn`](../reference/rules/ambiguous-column.md) | a column that exists in several joined tables |
| [`missingFromClauseEntry`](../reference/rules/missing-from-clause-entry.md) | `t.col` without `t` in the `FROM` clause |
| [`insertColumnMismatch`](../reference/rules/insert-column-mismatch.md) | more values than target columns, or the other way round |
| [`operatorTypeMismatch`](../reference/rules/operator-type-mismatch.md) | `1 + now()` |
| [`functionArgumentMismatch`](../reference/rules/function-argument-mismatch.md) | `length(1)`, or a call that matches several overloads |
| [`invalidCast`](../reference/rules/invalid-cast.md) | `now()::int` |
| [`assignmentTypeMismatch`](../reference/rules/assignment-type-mismatch.md) | `INSERT INTO t (qty) VALUES (now())` |
| [`functionReturnTypeMismatch`](../reference/rules/function-return-type-mismatch.md) | a `LANGUAGE sql` function whose final statement doesn't return what the function declares |

## Configuration

The `typecheck` rules are configured like any other linter rule or group:

```json
{
  "linter": {
    "rules": { "invalidCast": "warn" },
    "groups": { "typecheck": "error" }
  }
}
```

`typecheck.enabled: false` turns off both the `typecheck` rules and the `EXPLAIN` fallback.

You can configure the schemas included in the search path for type checking:

```json
{
  "typecheck": {
    "searchPath": ["public", "app_*", "auth"]
  }
}
```

The `searchPath` supports:
- Exact schema names (e.g., `"public"`)
- Glob patterns (e.g., `"app_*"` to match `app_users`, `app_products`, etc.)
- The order matters - schemas are searched in the order specified

Even if not specified, the LSP will always search`"public"` in last position. A `SET search_path` in the file takes effect for the statements after it.

## Requirements

Type checking requires an active database connection. Without one, the `typecheck` rules are skipped. The `EXPLAIN` fallback also needs permission to prepare statements in your database.
