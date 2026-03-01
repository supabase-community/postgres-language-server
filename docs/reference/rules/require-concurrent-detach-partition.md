# requireConcurrentDetachPartition
**Diagnostic Category: `lint/safety/requireConcurrentDetachPartition`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/detach-partition</code></a>

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
code-block.sql:1:1 lint/safety/requireConcurrentDetachPartition ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "safety": {
        "requireConcurrentDetachPartition": "error"
      }
    }
  }
}

```
