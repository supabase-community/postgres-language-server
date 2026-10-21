# renamingObject

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `Unreleased`  
**Diagnostic** `lint/renamingObject`

**Sources**: inspired by [`squawk/renaming-object`](https://squawkhq.com/docs/renaming-object)

## Description
Renaming an object or an enum value may break existing clients.

Clients that still use the old name of a view, function, type, sequence, schema,
constraint, role, or any other object fail after the rename. The same goes for enum values
renamed with `ALTER TYPE ... RENAME VALUE`.

Tables and columns are covered by `renamingTable` and `renamingColumn`.

## Examples

### Invalid

```sql
alter view active_users rename to enabled_users;
```

```sh
code-block.sql:1:1 lint/renamingObject ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Renaming a view may break existing clients.
  
  > 1 │ alter view active_users rename to enabled_users;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that still use the old name fail after the rename.
  
  i Update clients to use the new name before renaming.
  

```

```sql
alter type order_status rename value 'shipped' to 'dispatched';
```

```sh
code-block.sql:1:1 lint/renamingObject ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Renaming an enum value may break existing clients.
  
  > 1 │ alter type order_status rename value 'shipped' to 'dispatched';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Clients that still use the old name fail after the rename.
  
  i Update clients to use the new name before renaming.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "renamingObject": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore renamingObject
```
