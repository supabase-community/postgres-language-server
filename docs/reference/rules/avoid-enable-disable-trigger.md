# avoidEnableDisableTrigger

**Group** [`safety`](../rules.md#safety) · **Migrations only** · **Since** `0.25.0`  
**Diagnostic** `lint/avoidEnableDisableTrigger`

**Sources**: inspired by [`pgfence/enable-disable-trigger`](https://github.com/flvmnt/pgfence)

## Description
Enabling or disabling a trigger acquires a `SHARE ROW EXCLUSIVE` lock.

`ALTER TABLE ... ENABLE/DISABLE TRIGGER` blocks concurrent writes while the lock is held.
This can cause downtime on busy tables.

## Examples

### Invalid

```sql
alter table my_table enable trigger my_trigger;
```

```sh
code-block.sql:1:1 lint/avoidEnableDisableTrigger ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Enabling or disabling a trigger acquires a SHARE ROW EXCLUSIVE lock.
  
  > 1 │ alter table my_table enable trigger my_trigger;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i This blocks concurrent writes. Consider the impact on busy tables and use SET lock_timeout.
  

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
      "avoidEnableDisableTrigger": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore avoidEnableDisableTrigger
```
