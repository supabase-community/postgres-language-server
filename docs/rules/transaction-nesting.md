# transactionNesting
**Diagnostic Category: `lint/safety/transactionNesting`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/transaction-nesting" target="_blank"><code>squawk/transaction-nesting</code></a>

## Description
Detects problematic transaction nesting that could lead to unexpected behavior.

Transaction nesting issues occur when trying to start a transaction within an existing transaction,
or trying to commit/rollback when not in a transaction. This can lead to unexpected behavior
or errors in database migrations.

## Examples

### Invalid

```sql
BEGIN;
-- Migration tools already manage transactions
SELECT 1;
```

```sh
code-block.sql:1:1 lint/safety/transactionNesting ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Transaction already managed by migration tool.
  
  > 1 │ BEGIN;
      │ ^^^^^^
    2 │ -- Migration tools already manage transactions
    3 │ SELECT 1;
  
  i Migration tools manage transactions automatically. Remove explicit transaction control.
  
  i Put migration statements in separate files to have them be in separate transactions.
  

```

```sql
SELECT 1;
COMMIT; -- No transaction to commit
```

```sh
code-block.sql:2:1 lint/safety/transactionNesting ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Attempting to end transaction managed by migration tool.
  
    1 │ SELECT 1;
  > 2 │ COMMIT; -- No transaction to commit
      │ ^^^^^^^
    3 │ 
  
  i Migration tools manage transactions automatically. Remove explicit transaction control.
  
  i Put migration statements in separate files to have them be in separate transactions.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "transactionNesting": "error"
      }
    }
  }
}

```
