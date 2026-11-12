# addingNotNullField

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/addingNotNullField`

**Sources**: inspired by [`squawk/adding-not-null-field`](https://squawkhq.com/docs/adding-not-null-field)

## Description
Setting a column NOT NULL blocks reads while the table is scanned.

Setting NOT NULL on an existing column scans the table under an ACCESS EXCLUSIVE lock,
blocking reads and writes. This applies to `ALTER COLUMN ... SET NOT NULL` and, on
Postgres 18, to adding a NOT NULL constraint with `ADD CONSTRAINT ... NOT NULL` without
`NOT VALID`.

On Postgres 18, add the NOT NULL constraint as NOT VALID, then validate it in a separate
transaction. Validating only takes a SHARE UPDATE EXCLUSIVE lock, so reads and writes can
continue.

Before Postgres 18, a validated CHECK (column IS NOT NULL) constraint lets SET NOT NULL
skip the scan (Postgres 12+). Add the CHECK constraint as NOT VALID and validate it in a
separate transaction first.

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

```sql
ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null NOT NULL "foo";
```

```sh
code-block.sql:1:1 lint/addingNotNullField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Setting a column NOT NULL blocks reads while the table is scanned.
  
  > 1 │ ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null NOT NULL "foo";
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This operation requires an ACCESS EXCLUSIVE lock and a full table scan to verify all rows.
  
  i Add the constraint with ADD CONSTRAINT ... NOT NULL ... NOT VALID, then VALIDATE it in a separate transaction.
  

```

### Valid

On Postgres 18:

```sql
-- First add the NOT NULL constraint as NOT VALID
ALTER TABLE "core_recipe" ADD CONSTRAINT foo_not_null NOT NULL "foo" NOT VALID;
-- Then validate it in a separate transaction
ALTER TABLE "core_recipe" VALIDATE CONSTRAINT foo_not_null;
```

Before Postgres 18:

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
