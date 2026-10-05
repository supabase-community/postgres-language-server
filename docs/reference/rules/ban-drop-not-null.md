# banDropNotNull

**Group** [`destructive`](../rules.md#destructive) · **Recommended** · **Migrations only** · **Since** `0.1.0`  
**Diagnostic** `lint/banDropNotNull`

**Sources**: inspired by [`squawk/ban-drop-not-null`](https://squawkhq.com/docs/ban-drop-not-null)

## Description
Dropping a NOT NULL constraint may break existing clients.

Application code or code written in procedural languages like PL/SQL or PL/pgSQL may not expect NULL values for the column that was previously guaranteed to be NOT NULL and therefore may fail to process them correctly.

You can consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values.

## Examples

### Invalid

```sql
alter table users alter column email drop not null;
```

```sh
code-block.sql:1:1 lint/banDropNotNull ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a NOT NULL constraint may break existing clients.
  
  > 1 │ alter table users alter column email drop not null;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropNotNull": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropNotNull
```
