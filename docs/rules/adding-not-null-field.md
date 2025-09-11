# addingNotNullField
**Diagnostic Category: `lint/safety/addingNotNullField`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/adding-not-null-field" target="_blank"><code>squawk/adding-not-null-field</code></a>

## Description
Setting a column NOT NULL blocks reads while the table is scanned.

In PostgreSQL versions before 11, adding a NOT NULL constraint to an existing column requires
a full table scan to verify that all existing rows satisfy the constraint. This operation
takes an ACCESS EXCLUSIVE lock, blocking all reads and writes.

In PostgreSQL 11+, this operation is much faster as it can skip the full table scan for
newly added columns with default values.

Instead of using SET NOT NULL, consider using a CHECK constraint with NOT VALID, then
validating it in a separate transaction. This allows reads and writes to continue.

## Examples

### Invalid

```sql
ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
```

```sh
code-block.sql:1:1 lint/safety/addingNotNullField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Setting a column NOT NULL blocks reads while the table is scanned.
  
  > 1 │ ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.
  
  i Use a CHECK constraint with NOT VALID instead, then validate it in a separate transaction.
  

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
      "safety": {
        "addingNotNullField": "error"
      }
    }
  }
}

```
