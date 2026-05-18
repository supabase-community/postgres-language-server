# banVacuumFull
**Diagnostic Category: `lint/safety/banVacuumFull`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/vacuum-full</code></a>

## Description
`VACUUM FULL` rewrites the entire table and acquires an `ACCESS EXCLUSIVE` lock.

This blocks all reads and writes for the duration of the operation, which can
take a very long time on large tables. Use regular `VACUUM` or `pg_repack` instead
for online table maintenance.

## Examples

### Invalid

```sql
vacuum full my_table;
```

```sh
code-block.sql:1:1 lint/safety/banVacuumFull ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × VACUUM FULL rewrites the entire table and blocks all access.
  
  > 1 │ vacuum full my_table;
      │ ^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use regular VACUUM or pg_repack for online table maintenance without blocking reads and writes.
  

```

### Valid

```sql
vacuum my_table;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banVacuumFull": "error"
      }
    }
  }
}

```
