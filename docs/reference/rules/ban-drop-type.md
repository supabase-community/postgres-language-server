# banDropType

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banDropType` · **Postgres error** `42704`

**Sources**: inspired by [`squawk/ban-drop-type`](https://squawkhq.com/docs/ban-drop-type)

## Description
Dropping a type, cast, operator, operator class, or operator family may break existing clients.

Casts and parameters that name a dropped type fail with `42704 undefined_object`. With
`CASCADE`, columns of that type are dropped too.

Update your application code to no longer use the type, then drop it in a later migration.

## Examples

### Invalid

```sql
drop type order_status;
```

```sh
code-block.sql:1:1 lint/banDropType ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a type may break existing clients.
  
  > 1 │ drop type order_status;
      │ ^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Queries that still use it fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer use it, then drop it in a later migration.
  

```

```sql
drop cast (text as point);
```

```sh
code-block.sql:1:1 lint/banDropType ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a cast may break existing clients.
  
  > 1 │ drop cast (text as point);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Queries that still use it fail, and rolling back the application does not restore it.
  
  i Update your application code to no longer use it, then drop it in a later migration.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropType": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropType
```
