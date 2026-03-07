# banCreateTrigger
**Diagnostic Category: `lint/safety/banCreateTrigger`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/create-trigger</code></a>

## Description
Creating a trigger acquires a `SHARE ROW EXCLUSIVE` lock on the table.

`CREATE TRIGGER` can block concurrent writes while the lock is held.
Triggers also add hidden complexity to write operations on the table,
which can cause unexpected performance issues and make debugging harder.

## Examples

### Invalid

```sql
create trigger my_trigger after insert on my_table for each row execute function my_func();
```

```sh
code-block.sql:1:1 lint/safety/banCreateTrigger ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Creating a trigger acquires a SHARE ROW EXCLUSIVE lock on the table.
  
  > 1 │ create trigger my_trigger after insert on my_table for each row execute function my_func();
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Triggers add hidden complexity and can block concurrent writes. Consider using application-level logic instead.
  

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
        "banCreateTrigger": "error"
      }
    }
  }
}

```
