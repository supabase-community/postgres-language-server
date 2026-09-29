# banDropDatabase
**Diagnostic Category: `lint/banDropDatabase`**

**Group: `destructive`**

**Applies to: migration files only**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-drop-database" target="_blank"><code>squawk/ban-drop-database</code></a>

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
