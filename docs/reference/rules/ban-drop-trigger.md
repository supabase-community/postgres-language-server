# banDropTrigger

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/banDropTrigger`

**Sources**: inspired by [`pgfence/drop-trigger`](https://github.com/flvmnt/pgfence)

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
code-block.sql:1:1 lint/banDropTrigger ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
      "banDropTrigger": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banDropTrigger
```
