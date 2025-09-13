# addingForeignKeyConstraint
**Diagnostic Category: `lint/safety/addingForeignKeyConstraint`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/adding-foreign-key-constraint" target="_blank"><code>squawk/adding-foreign-key-constraint</code></a>

## Description
Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes.

Adding a foreign key constraint to an existing table can cause downtime by locking both tables while
verifying the constraint. PostgreSQL needs to check that all existing values in the referencing
column exist in the referenced table.

Instead, add the constraint as NOT VALID in one transaction, then VALIDATE it in another transaction.
This approach only takes a SHARE UPDATE EXCLUSIVE lock when validating, allowing concurrent writes.

## Examples

### Invalid

```sql
ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
```

```sh
code-block.sql:1:1 lint/safety/addingForeignKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a foreign key constraint requires a table scan and locks on both tables.
  
  > 1 │ ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This will block writes to both the referencing and referenced tables while PostgreSQL verifies the constraint.
  
  i Add the constraint as NOT VALID first, then VALIDATE it in a separate transaction.
  

```

```sql
ALTER TABLE "emails" ADD COLUMN "user_id" INT REFERENCES "user" ("id");
```

```sh
code-block.sql:1:1 lint/safety/addingForeignKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with a foreign key constraint requires a table scan and locks.
  
  > 1 │ ALTER TABLE "emails" ADD COLUMN "user_id" INT REFERENCES "user" ("id");
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Using REFERENCES when adding a column will block writes while verifying the constraint.
  
  i Add the column without the constraint first, then add the constraint as NOT VALID and VALIDATE it separately.
  

```

### Valid

```sql
-- First add the constraint as NOT VALID
ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id") NOT VALID;
-- Then validate it in a separate transaction
ALTER TABLE "email" VALIDATE CONSTRAINT "fk_user";
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "addingForeignKeyConstraint": "error"
      }
    }
  }
}

```
