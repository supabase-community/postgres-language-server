# banSetSchema

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/banSetSchema`

**Sources**: inspired by [`squawk/ban-set-schema`](https://squawkhq.com/docs/ban-set-schema)

## Description
Moving an object to another schema may break existing clients.

Clients that use the old schema-qualified name, like `public.users`, fail after the
object moved. Rolling back the application does not move the object back.

Update clients to use the new name, and keep the object at its old name until no running
application revision depends on it.

## Examples

### Invalid

```sql
alter table users set schema auth;
```

```sh
code-block.sql:1:1 lint/banSetSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Moving a table to another schema may break existing clients.
  
  > 1 │ alter table users set schema auth;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that use the old schema-qualified name fail, and rolling back the application does not move it back.
  
  i Update clients to use the new schema-qualified name before moving the object.
  

```

```sql
alter function calculate_total(int) set schema billing;
```

```sh
code-block.sql:1:1 lint/banSetSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Moving a function to another schema may break existing clients.
  
  > 1 │ alter function calculate_total(int) set schema billing;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that use the old schema-qualified name fail, and rolling back the application does not move it back.
  
  i Update clients to use the new schema-qualified name before moving the object.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banSetSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banSetSchema
```
