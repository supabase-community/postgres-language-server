# banDropColumn
**Diagnostic Category: `lint/safety/banDropColumn`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-drop-column" target="_blank"><code>squawk/ban-drop-column</code></a>

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
code-block.sql:1:1 lint/safety/banDropColumn ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "safety": {
        "banDropColumn": "error"
      }
    }
  }
}

```
