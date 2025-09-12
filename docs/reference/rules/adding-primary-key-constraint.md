# addingPrimaryKeyConstraint
**Diagnostic Category: `lint/safety/addingPrimaryKeyConstraint`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/adding-serial-primary-key-field" target="_blank"><code>squawk/adding-serial-primary-key-field</code></a>

## Description
Adding a primary key constraint results in locks and table rewrites.

When you add a PRIMARY KEY constraint, PostgreSQL needs to scan the entire table
to verify uniqueness and build the underlying index. This requires an ACCESS EXCLUSIVE
lock which blocks all reads and writes.

## Examples

### Invalid

```sql
ALTER TABLE users ADD PRIMARY KEY (id);
```

```sh
code-block.sql:1:1 lint/safety/addingPrimaryKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
code-block.sql:1:1 lint/safety/addingPrimaryKeyConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "safety": {
        "addingPrimaryKeyConstraint": "error"
      }
    }
  }
}

```
