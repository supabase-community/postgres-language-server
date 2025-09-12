# banConcurrentIndexCreationInTransaction
**Diagnostic Category: `lint/safety/banConcurrentIndexCreationInTransaction`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-concurrent-index-creation-in-transaction" target="_blank"><code>squawk/ban-concurrent-index-creation-in-transaction</code></a>

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
      "safety": {
        "banConcurrentIndexCreationInTransaction": "error"
      }
    }
  }
}

```
