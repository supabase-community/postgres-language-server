# multipleAlterTable

**Group** [`safety`](../rules.md#safety) · **Recommended** · **Migrations only** · **Since** `0.17.0`  
**Diagnostic** `lint/multipleAlterTable`

**Sources**: inspired by [`eugene/W12`](https://kaveland.no/eugene/hints/W12/index.html)

## Description
Multiple ALTER TABLE statements on the same table should be combined into a single statement.

When you run multiple ALTER TABLE statements on the same table, Postgres must scan and potentially
rewrite the table multiple times. Each ALTER TABLE command requires acquiring locks and performing
table operations that can be expensive, especially on large tables.

Combining multiple ALTER TABLE operations into a single statement with comma-separated actions
allows Postgres to scan and modify the table only once, improving performance and reducing
the time locks are held.

## Examples

### Invalid

```sql
ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
```

```sh
code-block.sql:2:1 lint/multipleAlterTable ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Multiple ALTER TABLE statements found for table public.authors.
  
    1 │ ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
  > 2 │ ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i Multiple ALTER TABLE statements on the same table require scanning and potentially rewriting the table multiple times.
  
  i Combine the ALTER TABLE statements into a single statement with comma-separated actions to scan the table only once.
  

```

### Valid

```sql
ALTER TABLE authors
  ALTER COLUMN name SET NOT NULL,
  ALTER COLUMN email SET NOT NULL;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "multipleAlterTable": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore multipleAlterTable
```
