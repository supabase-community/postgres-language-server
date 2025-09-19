# requireConcurrentIndexDeletion
**Diagnostic Category: `lint/safety/requireConcurrentIndexDeletion`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/require-concurrent-index-deletion" target="_blank"><code>squawk/require-concurrent-index-deletion</code></a>

## Description
Dropping indexes non-concurrently can lock the table for reads.

When dropping an index, using DROP INDEX without CONCURRENTLY will lock the table
preventing reads and writes for the duration of the drop. This can cause downtime in production systems.
Use DROP INDEX CONCURRENTLY to drop the index without blocking concurrent operations.

## Examples

### Invalid

```sql
DROP INDEX IF EXISTS users_email_idx;
```

```sh
code-block.sql:1:1 lint/safety/requireConcurrentIndexDeletion ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "safety": {
        "requireConcurrentIndexDeletion": "error"
      }
    }
  }
}

```
