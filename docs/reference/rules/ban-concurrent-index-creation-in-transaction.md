# banConcurrentIndexCreationInTransaction

**Group** [`correctness`](../rules.md#correctness) · **Recommended** · **Since** `0.15.0`  
**Diagnostic** `lint/banConcurrentIndexCreationInTransaction`

**Sources**: inspired by [`squawk/ban-concurrent-index-creation-in-transaction`](https://squawkhq.com/docs/ban-concurrent-index-creation-in-transaction)

## Description
Concurrent index creation is not allowed within a transaction.

`CREATE INDEX CONCURRENTLY` cannot be used within a transaction block. This will cause an error in Postgres.

Migration tools usually run each migration in a transaction, so using `CREATE INDEX CONCURRENTLY` will fail in such tools.

## Examples

### Invalid

```sql
CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
```

```sh
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
