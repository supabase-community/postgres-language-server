# lockTimeoutWarning

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.17.0`  
**Diagnostic** `lint/lockTimeoutWarning`

**Sources**: inspired by [`eugene/E9`](https://kaveland.no/eugene/hints/E9/index.html)

## Description
Taking a dangerous lock without setting a lock timeout can cause indefinite blocking.

When a statement acquires a lock that would block common operations (like SELECT, INSERT, UPDATE, DELETE),
it can cause the database to become unresponsive if another transaction is holding a conflicting lock
while idle in transaction or active. This is particularly dangerous for:

- ALTER TABLE statements (acquire ACCESS EXCLUSIVE lock)
- CREATE INDEX without CONCURRENTLY (acquires SHARE lock)

Setting a lock timeout ensures that if the lock cannot be acquired within a reasonable time,
the statement will fail rather than blocking indefinitely.

## Examples

### Invalid

```sql
ALTER TABLE users ADD COLUMN email TEXT;
```

```sh
code-block.sql:1:1 lint/lockTimeoutWarning ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Statement takes ACCESS EXCLUSIVE lock on public.users without lock timeout set.
  
  > 1 │ ALTER TABLE users ADD COLUMN email TEXT;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This can block all operations on the table indefinitely if another transaction holds a conflicting lock.
  
  i Run 'SET LOCAL lock_timeout = '2s';' before this statement and retry the migration if it times out.
  

```

```sql
CREATE INDEX users_email_idx ON users(email);
```

```sh
code-block.sql:1:1 lint/lockTimeoutWarning ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Statement takes SHARE lock on public.users while creating index users_email_idx without lock timeout set.
  
  > 1 │ CREATE INDEX users_email_idx ON users(email);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This blocks writes to the table indefinitely if another transaction holds a conflicting lock.
  
  i Run 'SET LOCAL lock_timeout = '2s';' before this statement, or use CREATE INDEX CONCURRENTLY to avoid blocking writes.
  

```

### Valid

```sql
-- Use CONCURRENTLY to avoid taking dangerous locks
CREATE INDEX CONCURRENTLY users_email_idx ON users(email);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "lockTimeoutWarning": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore lockTimeoutWarning
```
