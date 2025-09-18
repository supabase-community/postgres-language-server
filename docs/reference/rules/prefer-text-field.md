# preferTextField
**Diagnostic Category: `lint/safety/preferTextField`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/prefer-text-field" target="_blank"><code>squawk/prefer-text-field</code></a>

## Description
Prefer using TEXT over VARCHAR(n) types.

Changing the size of a VARCHAR field requires an ACCESS EXCLUSIVE lock, which blocks all
reads and writes to the table. It's easier to update a check constraint on a TEXT field
than a VARCHAR() size since the check constraint can use NOT VALID with a separate
VALIDATE call.

## Examples

### Invalid

```sql
CREATE TABLE "core_bar" (
    "id" serial NOT NULL PRIMARY KEY,
    "alpha" varchar(100) NOT NULL
);
```

```sh
code-block.sql:1:1 lint/safety/preferTextField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the size of a varchar field requires an ACCESS EXCLUSIVE lock.
  
  > 1 │ CREATE TABLE "core_bar" (
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^
  > 2 │     "id" serial NOT NULL PRIMARY KEY,
  > 3 │     "alpha" varchar(100) NOT NULL
  > 4 │ );
      │ ^^
    5 │ 
  
  i Use a text field with a check constraint.
  

```

```sql
ALTER TABLE "core_bar" ALTER COLUMN "kind" TYPE varchar(1000) USING "kind"::varchar(1000);
```

```sh
code-block.sql:1:1 lint/safety/preferTextField ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Changing the size of a varchar field requires an ACCESS EXCLUSIVE lock.
  
  > 1 │ ALTER TABLE "core_bar" ALTER COLUMN "kind" TYPE varchar(1000) USING "kind"::varchar(1000);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use a text field with a check constraint.
  

```

### Valid

```sql
CREATE TABLE "core_bar" (
    "id" serial NOT NULL PRIMARY KEY,
    "bravo" text NOT NULL
);
ALTER TABLE "core_bar" ADD CONSTRAINT "text_size" CHECK (LENGTH("bravo") <= 100);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "preferTextField": "error"
      }
    }
  }
}

```
