# renamingTable
**Diagnostic Category: `lint/safety/renamingTable`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/renaming-table" target="_blank"><code>squawk/renaming-table</code></a>

## Description
Renaming tables may break existing queries and application code.

Renaming a table that is being referenced by existing applications, views, functions, or foreign keys
can cause unexpected downtime. Consider creating a view with the old table name pointing to the new table,
or carefully coordinate the rename with application deployments.

## Examples

### Invalid

```sql
ALTER TABLE users RENAME TO app_users;
```

```sh
code-block.sql:1:1 lint/safety/renamingTable ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Renaming a table may break existing clients.
  
  > 1 │ ALTER TABLE users RENAME TO app_users;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider creating a view with the old table name instead, or coordinate the rename carefully with application deployments.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "renamingTable": "error"
      }
    }
  }
}

```
