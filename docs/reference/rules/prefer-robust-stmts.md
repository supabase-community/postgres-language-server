# preferRobustStmts
**Diagnostic Category: `lint/safety/preferRobustStmts`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-robust-stmts" target="_blank"><code>squawk/prefer-robust-stmts</code></a>

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
```

```sql
DROP INDEX CONCURRENTLY users_email_idx;
```

```sh
```

### Valid

```sql
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
```

```sql
DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferRobustStmts": "error"
      }
    }
  }
}

```
