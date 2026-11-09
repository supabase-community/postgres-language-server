# changingColumnType

**Group** [`destructive`](../rules.md#destructive) · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/changingColumnType`

**Sources**: inspired by [`squawk/changing-column-type`](https://squawkhq.com/docs/changing-column-type)

## Description
Changing a column type may require a table rewrite and break existing clients.

Most column type changes require an exclusive lock on the table while the entire
table is rewritten. This can take a long time for large tables and will block
reads and writes.

A type change doesn't rewrite the table when the old type is binary coercible to the new
one, like `varchar` to `text`, or when the column keeps its type and only drops a type
modifier, like `numeric(10,2)` to `numeric`. A `USING` clause other than the column itself
rewrites the table. The old type comes from the database and the statements before; when
it isn't known, only changes to these types count as safe:

- `text`
- `varchar` without a length limit
- `numeric` without a precision

For unsafe type changes, consider creating a new column with the desired type,
migrating the data, and then dropping the old column.

## Examples

### Invalid

```sql
ALTER TABLE "core_recipe" ALTER COLUMN "count" TYPE bigint;
```

```sh
code-block.sql:1:1 lint/changingColumnType ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing a column type requires a table rewrite and blocks reads and writes.
  
  > 1 │ ALTER TABLE "core_recipe" ALTER COLUMN "count" TYPE bigint;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider creating a new column with the desired type, migrating data, and then dropping the old column.
  

```

### Valid

```sql
ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "changingColumnType": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore changingColumnType
```
