# avoidWideLockWindow
**Diagnostic Category: `lint/avoidWideLockWindow`**

**Group: `safety`**

**Applies to: migration files only**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://github.com/flvmnt/pgfence" target="_blank"><code>pgfence/wide-lock-window</code></a>

## Description
Acquiring ACCESS EXCLUSIVE locks on multiple tables widens the lock window.

When a transaction holds an ACCESS EXCLUSIVE lock on one table and acquires
another on a different table, both locks are held until the transaction commits.
This increases the chance of blocking concurrent operations and causing downtime.

Split the operations into separate transactions to minimize the lock window.

## Examples

### Invalid

Acquiring locks on multiple tables in the same transaction:

```sql
ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE orders ADD COLUMN total NUMERIC;
```

```sh
code-block.sql:2:1 lint/avoidWideLockWindow ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Acquiring a lock on public.orders while already holding ACCESS EXCLUSIVE locks on other tables.
  
    1 │ ALTER TABLE users ADD COLUMN email TEXT;
  > 2 │ ALTER TABLE orders ADD COLUMN total NUMERIC;
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ 
  
  i This widens the lock window. Split into separate transactions to minimize lock duration.
  

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
      "avoidWideLockWindow": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore avoidWideLockWindow
```
