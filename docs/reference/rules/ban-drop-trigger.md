# banDropTrigger
**Diagnostic Category: `lint/safety/banDropTrigger`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/drop-trigger</code></a>

## Description
Dropping a trigger acquires an `ACCESS EXCLUSIVE` lock on the table.

`DROP TRIGGER` blocks all reads and writes on the table while the lock is held.
It may also break application logic that depends on the trigger's behavior.

## Examples

### Invalid

```sql
drop trigger my_trigger on my_table;
```

```sh
code-block.sql:1:1 lint/safety/banDropTrigger ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a trigger acquires an ACCESS EXCLUSIVE lock on the table.
  
  > 1 │ drop trigger my_trigger on my_table;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This blocks all reads and writes. Ensure no application logic depends on the trigger before dropping it.
  

```

### Valid

```sql
select 1;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banDropTrigger": "error"
      }
    }
  }
}

```
