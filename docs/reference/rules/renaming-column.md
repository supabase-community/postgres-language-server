# renamingColumn
**Diagnostic Category: `lint/safety/renamingColumn`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/renaming-column" target="_blank"><code>squawk/renaming-column</code></a>

## Description
Renaming columns may break existing queries and application code.

Renaming a column that is being used by an existing application or query can cause unexpected downtime.
Consider creating a new column instead and migrating the data, then dropping the old column after ensuring
no dependencies exist.

## Examples

### Invalid

```sql
ALTER TABLE users RENAME COLUMN email TO email_address;
```

```sh
code-block.sql:1:1 lint/safety/renamingColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Renaming a column may break existing clients.
  
  > 1 │ ALTER TABLE users RENAME COLUMN email TO email_address;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider creating a new column with the desired name and migrating data instead.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "renamingColumn": "error"
      }
    }
  }
}

```
