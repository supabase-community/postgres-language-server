# requireConcurrentReindex
**Diagnostic Category: `lint/safety/requireConcurrentReindex`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/reindex-non-concurrent</code></a>

## Description
`REINDEX` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock on the table.

This blocks all reads and writes until the reindex completes. Use `REINDEX CONCURRENTLY`
to rebuild the index without blocking concurrent operations.

## Examples

### Invalid

```sql
reindex index my_index;
```

```sh
code-block.sql:1:1 lint/safety/requireConcurrentReindex ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! REINDEX without CONCURRENTLY blocks all table access.
  
  > 1 │ reindex index my_index;
      │ ^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use REINDEX CONCURRENTLY to rebuild the index without blocking reads and writes.
  

```

### Valid

```sql
reindex index concurrently my_index;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "requireConcurrentReindex": "error"
      }
    }
  }
}

```
