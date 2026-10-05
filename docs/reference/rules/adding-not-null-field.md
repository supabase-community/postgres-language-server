# addingNotNullField

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/addingNotNullField`

**Sources**: inspired by [`squawk/adding-not-null-field`](https://squawkhq.com/docs/adding-not-null-field)

## Description
Setting a column NOT NULL blocks reads while the table is scanned.

Setting NOT NULL on an existing column scans the table under an ACCESS EXCLUSIVE lock,
blocking reads and writes. On PostgreSQL 12+, a validated CHECK (column IS NOT NULL)
constraint allows PostgreSQL to skip this scan.

Instead of using SET NOT NULL, consider using a CHECK constraint with NOT VALID, then
validating it in a separate transaction. This allows reads and writes to continue.

## Examples

### Invalid

```sql
ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
```

```sh
code-block.sql:1:1 lint/addingNotNullField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Setting a column NOT NULL blocks reads while the table is scanned.
  
  > 1 │ ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.
  
  i On PostgreSQL 12+, a validated CHECK (column IS NOT NULL) constraint lets PostgreSQL skip the scan.
  

```

### Valid

```sql
-- First add a CHECK constraint as NOT VALID
ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null CHECK (foo IS NOT NULL) NOT VALID;
-- Then validate it in a separate transaction
ALTER TABLE "core_recipe" VALIDATE CONSTRAINT foo_not_null;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "addingNotNullField": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore addingNotNullField
```
