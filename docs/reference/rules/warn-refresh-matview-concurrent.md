# warnRefreshMatviewConcurrent
**Diagnostic Category: `lint/safety/warnRefreshMatviewConcurrent`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/refresh-matview-concurrent</code></a>

## Description
`REFRESH MATERIALIZED VIEW CONCURRENTLY` still acquires an `EXCLUSIVE` lock.

While concurrent refresh allows reads during the refresh, it still blocks DDL
and other write operations on the materialized view. On large views, this can
take a long time.

## Examples

### Invalid

```sql
refresh materialized view concurrently my_view;
```

```sh
code-block.sql:1:1 lint/safety/warnRefreshMatviewConcurrent ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! REFRESH MATERIALIZED VIEW CONCURRENTLY still acquires an EXCLUSIVE lock.
  
  > 1 │ refresh materialized view concurrently my_view;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Concurrent refresh allows reads but still blocks DDL and writes. Consider the impact on long-running refreshes.
  

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
        "warnRefreshMatviewConcurrent": "error"
      }
    }
  }
}

```
