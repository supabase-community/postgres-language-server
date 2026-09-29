# runningStatementWhileHoldingAccessExclusive
**Diagnostic Category: `lint/runningStatementWhileHoldingAccessExclusive`**

**Group: `safety`**

**Applies to: migration files only**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://kaveland.no/eugene/hints/E4/index.html" target="_blank"><code>eugene/E4</code></a>

## Description
Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access.

When a transaction acquires an ACCESS EXCLUSIVE lock (e.g., via ALTER TABLE), it blocks all other
operations on that table, including reads. Running additional statements in the same transaction
extends the duration the lock is held, potentially blocking all database access to that table.

This is particularly problematic because:

- The lock blocks all SELECT, INSERT, UPDATE, DELETE operations
- The lock is held for the entire duration of all subsequent statements
- Even simple queries like SELECT COUNT(\*) can significantly extend lock time

To minimize blocking, run the ALTER TABLE in its own transaction and execute
other operations in separate transactions.

## Examples

### Invalid

```sql
ALTER TABLE authors ADD COLUMN email TEXT;
SELECT COUNT(*) FROM authors;
```

```sh
code-block.sql:2:1 lint/runningStatementWhileHoldingAccessExclusive ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Running statement while holding ACCESS EXCLUSIVE lock.
  
    1 │ ALTER TABLE authors ADD COLUMN email TEXT;
  > 2 │ SELECT COUNT(*) FROM authors;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i This blocks all access to the table for the duration of this statement.
  
  i Run this statement in a separate transaction to minimize lock duration.
  

```

### Valid

```sql
-- Run ALTER TABLE alone, other queries in separate transactions
ALTER TABLE authors ADD COLUMN email TEXT;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "runningStatementWhileHoldingAccessExclusive": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore runningStatementWhileHoldingAccessExclusive
```
