# requireConcurrentReindex

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Safe fix** · **Since** `0.25.0`  
**Diagnostic** `lint/requireConcurrentReindex`

**Sources**: inspired by [`pgfence/reindex-non-concurrent`](https://github.com/flvmnt/pgfence)

## Description
`REINDEX` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock on the table.

This blocks all reads and writes until the reindex completes. Use `REINDEX CONCURRENTLY`
to rebuild the index without blocking concurrent operations.

The fix adds `CONCURRENTLY` where the statement surely runs outside a transaction block, and
not on system catalogs, which Postgres can't reindex concurrently. Without migration section
markers, we don't know whether the migration tool runs the file in a transaction: a file
with more than one statement gets no fix, and the fix is unsafe in a file with a single
statement.

## Examples

### Invalid

```sql
reindex index my_index;
```

```sh
code-block.sql:1:1 lint/requireConcurrentReindex  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
