# banDropColumn

**Group** [`destructive`](../rules.md#destructive) · **Recommended** · **Migrations only** · **Since** `0.1.0`  
**Diagnostic** `lint/banDropColumn`

**Sources**: inspired by [`squawk/ban-drop-column`](https://squawkhq.com/docs/ban-drop-column)

## Description
Dropping a column may break existing clients.

Update your application code to no longer read or write the column.

You can leave the column as nullable or delete the column once queries no longer select or modify the column.

## Examples

### Invalid

```sql
alter table test drop column id;
```

```sh
code-block.sql:1:1 lint/banDropColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a column may break existing clients.
  
  > 1 │ alter table test drop column id;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i You can leave the column as nullable or delete the column once queries no longer select or modify the column.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banDropColumn": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropColumn
```
