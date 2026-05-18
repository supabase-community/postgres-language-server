# banDeleteWithoutWhere
**Diagnostic Category: `lint/safety/banDeleteWithoutWhere`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/delete-without-where</code></a>

## Description
A `DELETE` statement without a `WHERE` clause will remove all rows from the table.

This is almost always unintentional in a migration or application context.
If you truly need to remove all rows, use `TRUNCATE` explicitly (and acknowledge
its implications), or add a `WHERE true` to signal intent.

## Examples

### Invalid

```sql
delete from my_table;
```

```sh
code-block.sql:1:1 lint/safety/banDeleteWithoutWhere ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! A DELETE without a WHERE clause will remove all rows from the table.
  
  > 1 │ delete from my_table;
      │ ^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Add a WHERE clause to limit which rows are deleted.
  

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
      "safety": {
        "banDeleteWithoutWhere": "error"
      }
    }
  }
}

```
