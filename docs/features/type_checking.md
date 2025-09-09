# Type Checking

The Postgres Language Server validates your SQL queries against your actual database schema. As you type, it checks that your tables exist, columns are spelled correctly, and data types match - just like Postgres would when executing the query.

## How it Works

When you write a SQL query, the language server:
1. Connects to your database
2. Asks Postgres to validate your query without running it
3. Shows any errors directly in your editor

Since it uses your actual database, you get the exact same validation that would happen at runtime - but instantly as you type.

## Supported Statements

Since we are using `EXPLAIN`, type checking is only available for DML statements:
- `SELECT` statements
- `INSERT` statements
- `UPDATE` statements
- `DELETE` statements
- Common Table Expressions (CTEs)

## Configuration

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

If not configured, defaults to `["public"]`.

## What Gets Checked

The type checker catches common SQL mistakes:

- **Typos in table and column names**: `SELECT user_naem FROM users` → "column 'user_naem' does not exist"
- **Type mismatches**: `WHERE user_id = 'abc'` when `user_id` is an integer
- **Missing tables**: `SELECT * FROM user` when the table is named `users`
- **Wrong number of columns**: `INSERT INTO users VALUES (1)` when the table has multiple required columns

## Requirements

Type checking requires:

- An active database connection
- Appropriate permissions to prepare statements in your database
