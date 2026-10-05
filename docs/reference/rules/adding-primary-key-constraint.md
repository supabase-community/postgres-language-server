# addingPrimaryKeyConstraint

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.15.0`  
**Diagnostic** `lint/addingPrimaryKeyConstraint`

**Sources**: inspired by [`squawk/adding-serial-primary-key-field`](https://squawkhq.com/docs/adding-serial-primary-key-field)

## Description
Adding a primary key constraint results in locks and table rewrites.

When you add a PRIMARY KEY constraint, Postgres needs to scan the entire table
to verify uniqueness and build the underlying index. This requires an ACCESS EXCLUSIVE
lock which blocks all reads and writes.

## Examples

### Invalid

```sql
ALTER TABLE users ADD PRIMARY KEY (id);
```

```sh
code-block.sql:1:1 lint/addingPrimaryKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a PRIMARY KEY constraint results in locks and table rewrites.
  
  > 1 │ ALTER TABLE users ADD PRIMARY KEY (id);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Adding a PRIMARY KEY constraint requires an ACCESS EXCLUSIVE lock which blocks reads.
  
  i Add the PRIMARY KEY constraint USING an index.
  

```

```sql
ALTER TABLE items ADD COLUMN id SERIAL PRIMARY KEY;
```

```sh
code-block.sql:1:1 lint/addingPrimaryKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a PRIMARY KEY constraint results in locks and table rewrites.
  
  > 1 │ ALTER TABLE items ADD COLUMN id SERIAL PRIMARY KEY;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Adding a PRIMARY KEY constraint requires an ACCESS EXCLUSIVE lock which blocks reads.
  
  i Add the PRIMARY KEY constraint USING an index.
  

```

### Valid

```sql
-- First, create a unique index concurrently
CREATE UNIQUE INDEX CONCURRENTLY items_pk ON items (id);
-- Then add the primary key using the index
ALTER TABLE items ADD CONSTRAINT items_pk PRIMARY KEY USING INDEX items_pk;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "addingPrimaryKeyConstraint": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore addingPrimaryKeyConstraint
```
