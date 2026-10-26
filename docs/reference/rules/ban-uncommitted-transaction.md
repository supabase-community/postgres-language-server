# banUncommittedTransaction

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Migrations only** · **Unsafe fix** · **Since** `Unreleased`  
**Diagnostic** `lint/banUncommittedTransaction`

**Sources**: inspired by [`squawk/ban-uncommitted-transaction`](https://squawkhq.com/docs/ban-uncommitted-transaction)

## Description
A transaction is started but never committed or rolled back.

If a migration starts a transaction with `BEGIN` and never ends it, the migration may run
successfully without the changes ever being applied: the session closes, and Postgres
rolls the transaction back.

The rule is silent in migration sections that the migration tool runs in a transaction
of its own, which commits the changes.

## Examples

### Invalid

```sql
begin;
create table users (id bigint);
```

```sh
code-block.sql:1:1 lint/banUncommittedTransaction  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Transaction is never committed or rolled back.
  
  > 1 │ begin;
      │ ^^^^^^
    2 │ create table users (id bigint);
    3 │ 
  
  i Postgres rolls back the open transaction when the session ends, so its changes are not applied.
  
  i Add a COMMIT or ROLLBACK statement to complete the transaction.
  

```

### Valid

```sql
begin;
create table users (id bigint);
commit;
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banUncommittedTransaction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banUncommittedTransaction
```
