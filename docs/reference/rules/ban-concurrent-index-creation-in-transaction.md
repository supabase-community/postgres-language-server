# banConcurrentIndexCreationInTransaction

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Since** `0.15.0`  
**Diagnostic** `lint/banConcurrentIndexCreationInTransaction`

**Sources**: inspired by [`squawk/ban-concurrent-index-creation-in-transaction`](https://squawkhq.com/docs/ban-concurrent-index-creation-in-transaction)

## Description
Concurrent index operations are not allowed within a transaction.

`CREATE INDEX CONCURRENTLY`, `DROP INDEX CONCURRENTLY`, `REINDEX CONCURRENTLY` and
`ALTER TABLE ... DETACH PARTITION ... CONCURRENTLY` cannot be used within a transaction
block, or in the body of a function. This will cause an error in Postgres.

Migration tools usually run each migration in a transaction, so using `CONCURRENTLY` will fail in such tools.

With `migrations.tool`, the rule knows whether the migration runs in a transaction:
dbmate's `-- migrate:up transaction:false` and goose's `-- +goose NO TRANSACTION` disable it.
dbmate still sends the whole section to Postgres as one query, which Postgres runs in a
transaction block if it has more than one statement.

## Examples

### Invalid

```sql
BEGIN;
CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
COMMIT;
```

```sh
code-block.sql:2:1 lint/banConcurrentIndexCreationInTransaction ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × CREATE INDEX CONCURRENTLY cannot be used inside a transaction block.
  
    1 │ BEGIN;
  > 2 │ CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    3 │ COMMIT;
    4 │ 
  
  i Run CREATE INDEX CONCURRENTLY outside of a transaction. Migration tools usually run in transactions, so you may need to run this statement in its own migration or manually.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "banConcurrentIndexCreationInTransaction": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore banConcurrentIndexCreationInTransaction
```
