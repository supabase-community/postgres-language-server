# requireConcurrentIndexDeletion

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Safe fix** · **Since** `0.15.0`  
**Diagnostic** `lint/requireConcurrentIndexDeletion`

**Sources**: inspired by [`squawk/require-concurrent-index-deletion`](https://squawkhq.com/docs/require-concurrent-index-deletion)

## Description
Dropping indexes non-concurrently can lock the table for reads.

When dropping an index, using DROP INDEX without CONCURRENTLY will lock the table
preventing reads and writes for the duration of the drop. This can cause downtime in production systems.
Use DROP INDEX CONCURRENTLY to drop the index without blocking concurrent operations.

The fix adds `CONCURRENTLY` where the statement surely runs outside a transaction block, and
not on the indexes of partitioned tables, which Postgres can't drop concurrently. Without
migration section markers, we don't know whether the migration tool runs the file in a
transaction: a file with more than one statement gets no fix, and the fix is unsafe in a
file with a single statement.

## Examples

### Invalid

```sql
DROP INDEX IF EXISTS users_email_idx;
```

```sh
code-block.sql:1:1 lint/requireConcurrentIndexDeletion  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping an index non-concurrently blocks reads and writes to the table.
  
  > 1 │ DROP INDEX IF EXISTS users_email_idx;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use DROP INDEX CONCURRENTLY to avoid blocking concurrent operations on the table.
  

```

### Valid

```sql
DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireConcurrentIndexDeletion": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireConcurrentIndexDeletion
```
