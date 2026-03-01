# banBlockingRefreshMatview
**Diagnostic Category: `lint/safety/banBlockingRefreshMatview`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/refresh-matview-blocking</code></a>

## Description
`REFRESH MATERIALIZED VIEW` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock.

This blocks all reads on the materialized view until the refresh completes.
Use `REFRESH MATERIALIZED VIEW CONCURRENTLY` to allow reads during the refresh.
Note: concurrent refresh requires a unique index on the materialized view.

## Examples

### Invalid

```sql
refresh materialized view my_view;
```

```sh
code-block.sql:1:1 lint/safety/banBlockingRefreshMatview ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! REFRESH MATERIALIZED VIEW without CONCURRENTLY blocks all reads.
  
  > 1 │ refresh materialized view my_view;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use REFRESH MATERIALIZED VIEW CONCURRENTLY to allow reads during the refresh. This requires a unique index on the view.
  

```

### Valid

```sql
refresh materialized view concurrently my_view;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banBlockingRefreshMatview": "error"
      }
    }
  }
}

```
