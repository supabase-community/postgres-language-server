# requireConcurrentIndexCreation
**Diagnostic Category: `lint/safety/requireConcurrentIndexCreation`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/require-concurrent-index-creation" target="_blank"><code>squawk/require-concurrent-index-creation</code></a>

## Description
Creating indexes non-concurrently can lock the table for writes.

When creating an index on an existing table, using CREATE INDEX without CONCURRENTLY will lock the table
against writes for the duration of the index build. This can cause downtime in production systems.
Use CREATE INDEX CONCURRENTLY to build the index without blocking concurrent operations.

## Examples

### Invalid

```sql
CREATE INDEX users_email_idx ON users (email);
```

```sh
code-block.sql:1:1 lint/safety/requireConcurrentIndexCreation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Creating an index non-concurrently blocks writes to the table.
  
  > 1 │ CREATE INDEX users_email_idx ON users (email);
      │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    2 │ 
  
  i Use CREATE INDEX CONCURRENTLY to avoid blocking concurrent operations on the table.
  

```

### Valid

```sql
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "requireConcurrentIndexCreation": "error"
      }
    }
  }
}

```
