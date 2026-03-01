# banDropSchema
**Diagnostic Category: `lint/safety/banDropSchema`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/drop-schema</code></a>

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
code-block.sql:1:1 lint/safety/banDropSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "safety": {
        "banDropSchema": "error"
      }
    }
  }
}

```
