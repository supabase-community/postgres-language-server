# requireConcurrentDetachPartition

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/requireConcurrentDetachPartition`

**Sources**: inspired by [`pgfence/detach-partition`](https://github.com/flvmnt/pgfence)

## Description
Detaching a partition without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock.

`ALTER TABLE ... DETACH PARTITION` without `CONCURRENTLY` blocks all reads and writes
on the parent table. Use `DETACH PARTITION ... CONCURRENTLY` (Postgres 14+) to
avoid blocking concurrent operations.

## Examples

### Invalid

```sql
alter table my_table detach partition my_partition;
```

```sh
code-block.sql:1:1 lint/requireConcurrentDetachPartition ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Detaching a partition without CONCURRENTLY blocks all table access.
  
  > 1 │ alter table my_table detach partition my_partition;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use DETACH PARTITION ... CONCURRENTLY (Postgres 14+) to avoid blocking reads and writes.
  

```

### Valid

```sql
alter table my_table detach partition my_partition concurrently;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireConcurrentDetachPartition": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireConcurrentDetachPartition
```
