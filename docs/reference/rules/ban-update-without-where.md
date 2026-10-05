# banUpdateWithoutWhere

**Group** [`destructive`](../rules.md#destructive) · **Recommended** · **Since** `0.25.0`  
**Diagnostic** `lint/banUpdateWithoutWhere` · **Postgres error** `WHERE`

**Sources**: inspired by [`pgfence/update-in-migration`](https://github.com/flvmnt/pgfence)

## Description
An `UPDATE` statement without a `WHERE` clause will modify all rows in the table.

This is almost always unintentional in a migration context and can cause data corruption.
If you truly need to update all rows, add a `WHERE true` to signal intent.

## Examples

### Invalid

```sql
update my_table set col = 'value';
```

```sh
code-block.sql:1:1 lint/banUpdateWithoutWhere ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! An UPDATE without a WHERE clause will modify all rows in the table.
  
  > 1 │ update my_table set col = 'value';
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add a WHERE clause to limit which rows are updated.
  

```

### Valid

```sql
update my_table set col = 'value' where id = 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banUpdateWithoutWhere": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banUpdateWithoutWhere
```
