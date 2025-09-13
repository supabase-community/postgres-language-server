# addingFieldWithDefault
**Diagnostic Category: `lint/safety/addingFieldWithDefault`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/adding-field-with-default" target="_blank"><code>squawk/adding-field-with-default</code></a>

## Description
Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock.

In PostgreSQL versions before 11, adding a column with a DEFAULT value causes a full table rewrite,
which holds an ACCESS EXCLUSIVE lock on the table and blocks all reads and writes.

In PostgreSQL 11+, this behavior was optimized for non-volatile defaults. However:

- Volatile default values (like random() or custom functions) still cause table rewrites
- Generated columns (GENERATED ALWAYS AS) always require table rewrites
- Non-volatile defaults are safe in PostgreSQL 11+

## Examples

### Invalid

```sql
ALTER TABLE "core_recipe" ADD COLUMN "foo" integer DEFAULT 10;
```

```sh
code-block.sql:1:1 lint/safety/addingFieldWithDefault ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with a DEFAULT value causes a table rewrite.
  
  > 1 │ ALTER TABLE "core_recipe" ADD COLUMN "foo" integer DEFAULT 10;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This operation requires an ACCESS EXCLUSIVE lock and rewrites the entire table.
  
  i Add the column without a default, then set the default in a separate statement.
  

```

### Valid

```sql
ALTER TABLE "core_recipe" ADD COLUMN "foo" integer;
ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET DEFAULT 10;
-- Then backfill and add NOT NULL constraint if needed
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "addingFieldWithDefault": "error"
      }
    }
  }
}

```
