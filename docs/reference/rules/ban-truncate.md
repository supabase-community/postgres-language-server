# banTruncate

**Group** [`destructive`](../rules.md#destructive) · **Recommended** · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/banTruncate`

**Sources**: inspired by [`pgfence/truncate`](https://github.com/flvmnt/pgfence)

## Description
Truncating a table removes all rows and can cause data loss in production.

`TRUNCATE` is a fast, non-transactional (in terms of row-level locking) way to remove
all data from a table. It acquires an `ACCESS EXCLUSIVE` lock and cannot be safely
rolled back in all scenarios. In a migration context, this is almost always a mistake.

## Examples

### Invalid

```sql
truncate my_table;
```

```sh
code-block.sql:1:1 lint/banTruncate ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Truncating a table removes all rows and can cause data loss.
  
  > 1 │ truncate my_table;
      │ ^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use DELETE with a WHERE clause instead, or ensure this is intentional and not part of a migration.
  

```

### Valid

```sql
delete from my_table where expired_at < now();
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banTruncate": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banTruncate
```
