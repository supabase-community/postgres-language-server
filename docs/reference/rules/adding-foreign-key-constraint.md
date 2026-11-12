# addingForeignKeyConstraint

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/addingForeignKeyConstraint`

**Sources**: inspired by [`squawk/adding-foreign-key-constraint`](https://squawkhq.com/docs/adding-foreign-key-constraint)

## Description
Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes.

Adding a foreign key constraint to an existing table can cause downtime by locking both tables while
verifying the constraint. Postgres needs to check that all existing values in the referencing
column exist in the referenced table.

Instead, add the constraint as NOT VALID in one transaction, then VALIDATE it in another transaction.
This approach only takes a SHARE UPDATE EXCLUSIVE lock when validating, allowing concurrent writes.

A `NOT ENFORCED` foreign key (Postgres 18) isn't checked, so adding one doesn't scan the table.
Making it `ENFORCED` later with `ALTER CONSTRAINT` checks all rows while holding an ACCESS EXCLUSIVE
lock, and can't be split into NOT VALID and VALIDATE.

## Examples

### Invalid

```sql
ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
```

```sh
code-block.sql:1:1 lint/addingForeignKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a foreign key constraint requires a table scan and locks on both tables.
  
  > 1 │ ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This will block writes to both the referencing and referenced tables while Postgres verifies the constraint.
  
  i Add the constraint as NOT VALID first, then VALIDATE it in a separate transaction.
  

```

```sql
ALTER TABLE "emails" ADD COLUMN "user_id" INT REFERENCES "user" ("id");
```

```sh
code-block.sql:1:1 lint/addingForeignKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with a foreign key constraint requires a table scan and locks.
  
  > 1 │ ALTER TABLE "emails" ADD COLUMN "user_id" INT REFERENCES "user" ("id");
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Using REFERENCES when adding a column will block writes while verifying the constraint.
  
  i Add the column without the constraint first, then add the constraint as NOT VALID and VALIDATE it separately.
  

```

```sql
ALTER TABLE "email" ALTER CONSTRAINT "fk_user" ENFORCED;
```

```sh
code-block.sql:1:1 lint/addingForeignKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Enforcing foreign key constraint fk_user checks all rows of the table.
  
  > 1 │ ALTER TABLE "email" ALTER CONSTRAINT "fk_user" ENFORCED;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This holds an ACCESS EXCLUSIVE lock on the table while Postgres verifies the constraint.
  
  i Drop the constraint and add it again as NOT VALID, then VALIDATE it in a separate transaction.
  

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
      "addingForeignKeyConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore addingForeignKeyConstraint
```
