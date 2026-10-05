# requireStatementTimeout

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/requireStatementTimeout`

**Sources**: inspired by [`pgfence/missing-statement-timeout`](https://github.com/flvmnt/pgfence)

## Description
Dangerous lock statements should be preceded by `SET statement_timeout`.

Long-running statements holding locks can block other operations. Setting a
`statement_timeout` ensures the statement fails rather than running indefinitely.

## Examples

### Invalid

```sql
ALTER TABLE users ADD COLUMN email TEXT;
```

```sh
code-block.sql:1:1 lint/requireStatementTimeout ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Statement takes a dangerous lock without a statement_timeout set.
  
  > 1 │ ALTER TABLE users ADD COLUMN email TEXT;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Run SET statement_timeout = '...' before this statement to prevent it from running indefinitely.
  

```

### Valid

```sql
CREATE INDEX CONCURRENTLY users_email_idx ON users(email);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireStatementTimeout": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireStatementTimeout
```
