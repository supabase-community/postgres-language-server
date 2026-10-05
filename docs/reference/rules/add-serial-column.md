# addSerialColumn

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.17.0`  
**Diagnostic** `lint/addSerialColumn`

**Sources**: inspired by [`eugene/E11`](https://kaveland.no/eugene/hints/E11/index.html)

## Description
Adding a column with a SERIAL type or GENERATED ALWAYS AS ... STORED causes a full table rewrite.

When adding a column with a SERIAL type (serial, bigserial, smallserial) or a GENERATED ALWAYS AS ... STORED column
to an existing table, Postgres must rewrite the entire table while holding an ACCESS EXCLUSIVE lock.
This blocks all reads and writes to the table for the duration of the rewrite operation.

SERIAL types are implemented using sequences and DEFAULT values, while GENERATED ... STORED columns require
computing and storing values for all existing rows. Both operations require rewriting every row in the table.

## Examples

### Invalid

```sql
ALTER TABLE prices ADD COLUMN id serial;
```

```sh
code-block.sql:1:1 lint/addSerialColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with type serial requires a table rewrite.
  
  > 1 │ ALTER TABLE prices ADD COLUMN id serial;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i SERIAL types require rewriting the entire table with an ACCESS EXCLUSIVE lock, blocking all reads and writes.
  
  i SERIAL types cannot be added to existing tables without a full table rewrite. Consider using a non-serial type with a sequence instead.
  

```

```sql
ALTER TABLE prices ADD COLUMN id bigserial;
```

```sh
code-block.sql:1:1 lint/addSerialColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with type bigserial requires a table rewrite.
  
  > 1 │ ALTER TABLE prices ADD COLUMN id bigserial;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i SERIAL types require rewriting the entire table with an ACCESS EXCLUSIVE lock, blocking all reads and writes.
  
  i SERIAL types cannot be added to existing tables without a full table rewrite. Consider using a non-serial type with a sequence instead.
  

```

```sql
ALTER TABLE prices ADD COLUMN total int GENERATED ALWAYS AS (price * quantity) STORED;
```

```sh
code-block.sql:1:1 lint/addSerialColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Adding a column with GENERATED ALWAYS AS ... STORED requires a table rewrite.
  
  > 1 │ ALTER TABLE prices ADD COLUMN total int GENERATED ALWAYS AS (price * quantity) STORED;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i GENERATED ... STORED columns require rewriting the entire table with an ACCESS EXCLUSIVE lock, blocking all reads and writes.
  
  i GENERATED ... STORED columns cannot be added to existing tables without a full table rewrite.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "addSerialColumn": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore addSerialColumn
```
