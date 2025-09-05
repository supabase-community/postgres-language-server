# banDropNotNull
**Diagnostic Category: `lint/safety/banDropNotNull`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-drop-not-null" target="_blank"><code>squawk/ban-drop-not-null</code></a>

## Description
Dropping a NOT NULL constraint may break existing clients.

Application code or code written in procedural languages like PL/SQL or PL/pgSQL may not expect NULL values for the column that was previously guaranteed to be NOT NULL and therefore may fail to process them correctly.

You can consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values.

## Examples

### Invalid

```sql
alter table users alter column email drop not null;
```

```sh
code-block.sql:1:1 lint/safety/banDropNotNull ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a NOT NULL constraint may break existing clients.
  
  > 1 │ alter table users alter column email drop not null;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Consider using a marker value that represents NULL. Alternatively, create a new table allowing NULL values, copy the data from the old table, and create a view that filters NULL values.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banDropNotNull": "error"
      }
    }
  }
}

```
