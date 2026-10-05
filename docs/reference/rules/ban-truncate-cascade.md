# banTruncateCascade

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `0.9.0`  
**Diagnostic** `lint/banTruncateCascade`

**Sources**: inspired by [`squawk/ban-truncate-cascade`](https://squawkhq.com/docs/ban-truncate-cascade)

## Description
Using `TRUNCATE`'s `CASCADE` option will truncate any tables that are also foreign-keyed to the specified tables.

So if you had tables with foreign-keys like:

`a <- b <- c`

and ran:

`truncate a cascade;`

You'd end up with a, b, & c all being truncated!

Instead, you can manually specify the tables you want.

`truncate a, b;`

## How to configure
```json

{
  "linter": {
    "rules": {
      "banTruncateCascade": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banTruncateCascade
```
