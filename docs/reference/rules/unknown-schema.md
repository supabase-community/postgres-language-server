# unknownSchema
**Diagnostic Category: `lint/unknownSchema`**

**Group: `typecheck`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

## Description
A schema does not exist.

`pg_catalog` and `pg_temp` always exist. Schemas created earlier in the same file are known.

Postgres raises `3F000 invalid_schema_name` for these statements.

## Examples

### Invalid

```sql
select * from missing_schema.users;
```

```sh
code-block.sql:1:15 lint/unknownSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Schema missing_schema does not exist.
  
  > 1 │ select * from missing_schema.users;
      │               ^^^^^^^^^^^^^^^^^^^^
    2 │ 
  

```

### Valid

```sql
create schema app;
create table app.users (id int8);
select * from app.users;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "unknownSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore unknownSchema
```
