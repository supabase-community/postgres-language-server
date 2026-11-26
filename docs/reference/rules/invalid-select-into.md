# invalidSelectInto

**Group** [`typecheck`](../rules.md#typecheck) · **Recommended** · **Needs a database connection** · **Since** `Unreleased`  
**Diagnostic** `lint/invalidSelectInto` · **Postgres error** `42601`

## Description
A `SELECT ... INTO` that Postgres rejects.

In a set operation like `UNION`, only the first select may have an `INTO`.

Postgres raises `42601 syntax_error`.

## Examples

### Invalid

```sql
select 1 as id union select 2 into ids;
```

```sh
code-block.sql:1:36 lint/invalidSelectInto ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × INTO is only allowed on first SELECT of UNION/INTERSECT/EXCEPT.
  
  > 1 │ select 1 as id union select 2 into ids;
      │                                    ^^^
    2 │ 
  

```

### Valid

```sql
select 1 as id into ids union select 2;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "invalidSelectInto": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore invalidSelectInto
```
