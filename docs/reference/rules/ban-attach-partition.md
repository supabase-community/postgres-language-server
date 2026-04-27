# banAttachPartition
**Diagnostic Category: `lint/safety/banAttachPartition`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/attach-partition</code></a>

## Description
Attaching a partition acquires an `ACCESS EXCLUSIVE` lock on the parent table.

`ALTER TABLE ... ATTACH PARTITION` locks the parent table, blocking all reads and writes.
For large tables, this can cause significant downtime. Consider creating the partition
with the correct constraints upfront, or use a staging table approach.

## Examples

### Invalid

```sql
alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');
```

```sh
code-block.sql:1:1 lint/safety/banAttachPartition ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Attaching a partition acquires an ACCESS EXCLUSIVE lock on the parent table.
  
  > 1 │ alter table my_table attach partition my_partition for values from ('2024-01-01') to ('2025-01-01');
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This blocks all reads and writes on the parent table. Consider adding a matching CHECK constraint to the child table before attaching to minimize lock duration.
  

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
      "safety": {
        "banAttachPartition": "error"
      }
    }
  }
}

```
