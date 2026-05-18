# requireIdleInTransactionTimeout
**Diagnostic Category: `lint/safety/requireIdleInTransactionTimeout`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/missing-idle-timeout</code></a>

## Description
Dangerous lock statements should be preceded by `SET idle_in_transaction_session_timeout`.

A transaction holding dangerous locks that goes idle (e.g., due to application errors)
will block other operations indefinitely. Setting `idle_in_transaction_session_timeout`
ensures the session is terminated if it sits idle too long.

## Examples

### Invalid

```sql
ALTER TABLE users ADD COLUMN email TEXT;
```

```sh
code-block.sql:1:1 lint/safety/requireIdleInTransactionTimeout ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Statement takes a dangerous lock without idle_in_transaction_session_timeout set.
  
  > 1 │ ALTER TABLE users ADD COLUMN email TEXT;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Run SET idle_in_transaction_session_timeout = '...' before this statement to prevent idle transactions from holding locks.
  

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
      "safety": {
        "requireIdleInTransactionTimeout": "error"
      }
    }
  }
}

```
