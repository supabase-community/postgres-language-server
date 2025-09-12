# disallowUniqueConstraint
**Diagnostic Category: `lint/safety/disallowUniqueConstraint`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/disallow-unique-constraint" target="_blank"><code>squawk/disallow-unique-constraint</code></a>

## Description
Disallow adding a UNIQUE constraint without using an existing index.

Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock, which blocks all reads and
writes to the table. Instead, create a unique index concurrently and then add the
constraint using that index.

## Examples

### Invalid

```sql
ALTER TABLE table_name ADD CONSTRAINT field_name_constraint UNIQUE (field_name);
```

```sh
code-block.sql:1:1 lint/safety/disallowUniqueConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock.
  
  > 1 │ ALTER TABLE table_name ADD CONSTRAINT field_name_constraint UNIQUE (field_name);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Create a unique index CONCURRENTLY and then add the constraint using that index.
  

```

```sql
ALTER TABLE foo ADD COLUMN bar text UNIQUE;
```

```sh
code-block.sql:1:1 lint/safety/disallowUniqueConstraint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Adding a UNIQUE constraint requires an ACCESS EXCLUSIVE lock.
  
  > 1 │ ALTER TABLE foo ADD COLUMN bar text UNIQUE;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Create a unique index CONCURRENTLY and then add the constraint using that index.
  

```

### Valid

```sql
CREATE UNIQUE INDEX CONCURRENTLY dist_id_temp_idx ON distributors (dist_id);
ALTER TABLE distributors DROP CONSTRAINT distributors_pkey,
ADD CONSTRAINT distributors_pkey PRIMARY KEY USING INDEX dist_id_temp_idx;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "disallowUniqueConstraint": "error"
      }
    }
  }
}

```
