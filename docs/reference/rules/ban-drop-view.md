# banDropView

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropView` · **Postgres error** `42P01`

**Sources**: inspired by [`squawk/ban-drop-view`](https://squawkhq.com/docs/ban-drop-view)

## Description
Dropping a view or materialized view may break existing clients.

Queries that still select from the view fail with `42P01 undefined_table`. Rolling back
the application does not bring the view back.

Update your application code to no longer use the view, then drop it in a later migration.

## Examples

### Invalid

```sql
drop view active_users;
```

```sh
code-block.sql:1:1 lint/banDropView ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a view may break existing clients.
  
  > 1 │ drop view active_users;
      │ ^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Queries that still use the view fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer use the view, then drop it in a later migration.
  

```

```sql
drop materialized view user_stats;
```

```sh
code-block.sql:1:1 lint/banDropView ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a materialized view may break existing clients.
  
  > 1 │ drop materialized view user_stats;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Queries that still use the view fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer use the view, then drop it in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropView": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropView
```
