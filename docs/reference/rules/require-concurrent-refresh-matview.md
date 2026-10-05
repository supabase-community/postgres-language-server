# requireConcurrentRefreshMatview

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/requireConcurrentRefreshMatview`

**Sources**: inspired by [`pgfence/refresh-matview-blocking`](https://github.com/flvmnt/pgfence)

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
code-block.sql:1:1 lint/requireConcurrentRefreshMatview ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "requireConcurrentRefreshMatview": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireConcurrentRefreshMatview
```
