# avoidAttachingPartition

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/avoidAttachingPartition`

**Sources**: inspired by [`pgfence/attach-partition`](https://github.com/flvmnt/pgfence)

## Description
Attaching a partition acquires an `ACCESS EXCLUSIVE` lock on the partition.

`ALTER TABLE ... ATTACH PARTITION` takes a `SHARE UPDATE EXCLUSIVE` lock on the parent
table, which doesn't block reads and writes, and an `ACCESS EXCLUSIVE` lock on the
partition and on the default partition, if there is one. While holding them, Postgres
scans the partition to check that its rows match the partition bound, and the default
partition to check that none of its rows do. For large tables, this can cause significant
downtime. Add a `CHECK` constraint that matches the partition bound to the partition
before attaching it, so that Postgres skips the scan.

## Examples

### Invalid

```sql
alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');
```

```sh
code-block.sql:1:1 lint/avoidAttachingPartition ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Attaching a partition acquires an ACCESS EXCLUSIVE lock on the partition and a SHARE UPDATE EXCLUSIVE lock on the parent table.
  
  > 1 │ alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This blocks all reads and writes on the partition, and on the default partition if there is one, while Postgres scans them to validate the partition bound. Add a CHECK constraint that matches the partition bound to the partition before attaching it, so that Postgres skips the scan.
  

```

### Valid

```sql
select 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "avoidAttachingPartition": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore avoidAttachingPartition
```
