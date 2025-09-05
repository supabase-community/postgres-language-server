# banDropDatabase
**Diagnostic Category: `lint/safety/banDropDatabase`**

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
      "safety": {
        "banDropDatabase": "error"
      }
    }
  }
}

```
