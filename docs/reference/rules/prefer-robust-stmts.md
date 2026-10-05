# preferRobustStmts

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/preferRobustStmts`

**Sources**: inspired by [`squawk/prefer-robust-stmts`](https://squawkhq.com/docs/prefer-robust-stmts)

## Description
Prefer statements with guards for robustness in migrations.

When running migrations outside of transactions (e.g., CREATE INDEX CONCURRENTLY),
statements should be made robust by using guards like IF NOT EXISTS or IF EXISTS.
This allows migrations to be safely re-run if they fail partway through.

## Examples

### Invalid

```sql
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
```

```sh
code-block.sql:1:1 lint/preferRobustStmts ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Concurrent index creation should use IF NOT EXISTS.
  
  > 1 │ CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add IF NOT EXISTS to make the migration re-runnable if it fails.
  

```

```sql
DROP INDEX CONCURRENTLY users_email_idx;
```

```sh
code-block.sql:1:1 lint/preferRobustStmts ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Concurrent drop should use IF EXISTS.
  
  > 1 │ DROP INDEX CONCURRENTLY users_email_idx;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add IF EXISTS to make the migration re-runnable if it fails.
  

```

```sql
CREATE TABLE users (id int);
```

```sh
code-block.sql:1:1 lint/preferRobustStmts ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! CREATE TABLE should use IF NOT EXISTS.
  
  > 1 │ CREATE TABLE users (id int);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add IF NOT EXISTS to make the migration re-runnable if it fails.
  

```

```sql
DROP TABLE users;
```

```sh
code-block.sql:1:1 lint/preferRobustStmts ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! DROP TABLE should use IF EXISTS.
  
  > 1 │ DROP TABLE users;
      │ ^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add IF EXISTS to make the migration re-runnable if it fails.
  

```

### Valid

```sql
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
```

```sql
DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
```

```sql
CREATE TABLE IF NOT EXISTS users (id int);
```

```sql
DROP TABLE IF EXISTS users;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "preferRobustStmts": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore preferRobustStmts
```
