# banDropDatabase

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `0.9.0`  
**Diagnostic** `lint/banDropDatabase`

**Sources**: inspired by [`squawk/ban-drop-database`](https://squawkhq.com/docs/ban-drop-database)

## Description
Dropping a database may break existing clients (and everything else, really).

Make sure that you really want to drop it.

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropDatabase": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropDatabase
```
