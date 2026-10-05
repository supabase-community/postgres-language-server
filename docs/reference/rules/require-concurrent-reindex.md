# requireConcurrentReindex

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/requireConcurrentReindex`

**Sources**: inspired by [`pgfence/reindex-non-concurrent`](https://github.com/flvmnt/pgfence)

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
code-block.sql:1:1 lint/requireConcurrentReindex ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "requireConcurrentReindex": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireConcurrentReindex
```
