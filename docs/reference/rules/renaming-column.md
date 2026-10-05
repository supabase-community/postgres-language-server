# renamingColumn

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/renamingColumn`

**Sources**: inspired by [`squawk/renaming-column`](https://squawkhq.com/docs/renaming-column)

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
code-block.sql:1:1 lint/renamingColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "renamingColumn": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore renamingColumn
```
