# changingColumnType
**Diagnostic Category: `lint/safety/changingColumnType`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/changing-column-type" target="_blank"><code>squawk/changing-column-type</code></a>

## Description
Changing a column type may break existing clients.

Changing a column's data type requires an exclusive lock on the table while the entire table is rewritten.
This can take a long time for large tables and will block reads and writes.

Instead of changing the type directly, consider creating a new column with the desired type,
migrating the data, and then dropping the old column.

## Examples

### Invalid

```sql
ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text USING "edits"::text;
```

```sh
code-block.sql:1:1 lint/safety/changingColumnType ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing a column type requires a table rewrite and blocks reads and writes.
  
  > 1 │ ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text USING "edits"::text;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider creating a new column with the desired type, migrating data, and then dropping the old column.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "changingColumnType": "error"
      }
    }
  }
}

```
