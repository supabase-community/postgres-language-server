# banDropSchema

**Group** [`destructive`](../rules.md#destructive) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/banDropSchema`

**Sources**: inspired by [`pgfence/drop-schema`](https://github.com/flvmnt/pgfence)

## Description
Dropping a schema will remove all objects within it and may break existing clients.

A `DROP SCHEMA` statement removes the entire schema and all objects it contains.
This is a destructive operation that can cause significant data loss and break
dependent applications.

## Examples

### Invalid

```sql
drop schema my_schema;
```

```sh
code-block.sql:1:1 lint/banDropSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Dropping a schema will remove all objects within it and may break existing clients.
  
  > 1 │ drop schema my_schema;
      │ ^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Remove objects individually instead, or ensure all dependent applications have been updated.
  

```

### Valid

```sql
select 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropSchema
```
